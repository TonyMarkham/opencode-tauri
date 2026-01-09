namespace OpenCode.Services;

using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Opencode;
using Opencode.Session;
using OpenCode.Services.Exceptions;
using Google.Protobuf;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text.Json;
using System.Net.WebSockets;

/// <summary>
/// Production-grade IPC client with thread-safe WebSocket management.
/// </summary>
public class IpcClient : IIpcClient, IDisposable
{
    private readonly IIpcConfigService _configService;
    private readonly ILogger<IpcClient> _logger;
    private readonly IpcClientOptions _options;
    private readonly IIpcClientMetrics _metrics;

    // WebSocket and state
    private ClientWebSocket? _ws;
    private ConnectionState _state = ConnectionState.Disconnected;
    private CancellationTokenSource? _cts;
    private Task? _receiveTask;

    // Thread safety
    private readonly SemaphoreSlim _connectionLock = new(1, 1);
    private readonly SemaphoreSlim _sendLock = new(1, 1);

    // Request/response correlation
    private long _nextRequestId = 1; // long for Interlocked.Increment
    private readonly ConcurrentDictionary<ulong, TaskCompletionSource<IpcServerMessage>> _pendingRequests = new();

    // Disposal
    private int _disposed = 0;
    
    // Shared JSON options for config deserialization (thread-safe, reusable)
    private static readonly JsonSerializerOptions s_jsonOptions = new()
    {
        PropertyNameCaseInsensitive = true
    };

    public event EventHandler<ConnectionStateChangedEventArgs>? ConnectionStateChanged;

    public bool IsConnected => _state == ConnectionState.Connected && _ws?.State == WebSocketState.Open;

    public IpcClient(
        IIpcConfigService configService,
        ILogger<IpcClient> logger,
        IOptions<IpcClientOptions> options,
        IIpcClientMetrics metrics)
    {
        _configService = configService;
        _logger = logger;
        _options = options.Value;
        _metrics = metrics;
    }

    // TODO: Step 6B - Will implement these methods
    /// <summary>
    /// Connects to IPC server with thread-safe state management.
    /// </summary>
    public async Task ConnectAsync()
    {
        await _connectionLock.WaitAsync();
        try
        {
            // Idempotent: already connected
            if (_state == ConnectionState.Connected)
            {
                _logger.LogDebug("Already connected, skipping ConnectAsync");
                return;
            }

            if (_state == ConnectionState.Connecting)
            {
                throw new InvalidOperationException("Connection already in progress");
            }

            SetConnectionState(ConnectionState.Connecting);

            try
            {
                // 1. Get config with validation
                _logger.LogInformation("Retrieving IPC configuration...");
                var (port, authToken) = await _configService.GetConfigAsync();

                // 2. Connect WebSocket
                var endpoint = $"ws://127.0.0.1:{port}";
                _logger.LogInformation("Connecting to {Endpoint}", endpoint);

                _ws = new ClientWebSocket();

                using var connectCts = new CancellationTokenSource(_options.ConnectionTimeout);
                await _ws.ConnectAsync(new Uri(endpoint), connectCts.Token);

                _logger.LogInformation("WebSocket connected");

                // 3. Start receive loop
                _cts = new CancellationTokenSource();
                _receiveTask = Task.Run(() => ReceiveLoopAsync(_cts.Token), _cts.Token);

                // 4. Authenticate
                SetConnectionState(ConnectionState.Authenticating);
                await AuthenticateAsync(authToken);

                SetConnectionState(ConnectionState.Connected);
                _logger.LogInformation("IPC client connected and authenticated");
            }
            catch (Exception ex)
            {
                SetConnectionState(ConnectionState.Failed);

                // Clean up partial connection
                _cts?.Cancel();
                _ws?.Dispose();
                _ws = null;
                _cts?.Dispose();
                _cts = null;

                _logger.LogError(ex, "Failed to connect to IPC server");
                throw;
            }
        }
        finally
        {
            _connectionLock.Release();
        }
    }

    /// <summary>
    /// Performs authentication handshake (request_id: 1).
    /// </summary>
    private async Task AuthenticateAsync(string token)
    {
        _logger.LogDebug("Sending authentication handshake");

        var authMsg = new IpcClientMessage
        {
            RequestId = 1,
            AuthHandshake = new IpcAuthHandshake { Token = token }
        };

        using var authCts = new CancellationTokenSource(_options.AuthenticationTimeout);
        var response = await SendRequestAsync(authMsg, requestId: 1, cancellationToken: authCts.Token);

        if (!response.AuthHandshakeResponse.Success)
        {
            var errorMsg = response.AuthHandshakeResponse.Error ?? "Authentication failed";
            _logger.LogError("Authentication failed: {Error}", errorMsg);
            throw new IpcAuthenticationException(errorMsg);
        }

        _logger.LogInformation("Authentication successful");
    }

    /// <summary>
    /// Sets connection state and raises event.
    /// </summary>
    private void SetConnectionState(ConnectionState newState)
    {
        var oldState = _state;
        _state = newState;

        if (oldState != newState)
        {
            _logger.LogInformation("Connection state: {OldState} -> {NewState}", oldState, newState);
            _metrics.RecordConnectionStateChange(oldState, newState);
            ConnectionStateChanged?.Invoke(this, new ConnectionStateChangedEventArgs(oldState, newState));
        }
    }

    /// <summary>
    /// Gets next request ID (thread-safe).
    /// </summary>
    private ulong GetNextRequestId()
    {
        var id = Interlocked.Increment(ref _nextRequestId);
        return (ulong)id;
    }

    /// <summary>
    /// Background receive loop handling fragmented WebSocket messages.
    /// </summary>
    private async Task ReceiveLoopAsync(CancellationToken ct)
    {
        _logger.LogDebug("Receive loop started");

        using var memoryStream = new MemoryStream();
        var buffer = new byte[_options.MaxReceiveBufferSize];

        while (!ct.IsCancellationRequested && _ws?.State == WebSocketState.Open)
        {
            try
            {
                memoryStream.SetLength(0); // Reset for new message
                WebSocketReceiveResult result;

                // Read all fragments of the message
                do
                {
                    result = await _ws.ReceiveAsync(new ArraySegment<byte>(buffer), ct);

                    if (result.MessageType == WebSocketMessageType.Close)
                    {
                        _logger.LogInformation("Server closed WebSocket connection: {Status} {Description}",
                            result.CloseStatus, result.CloseStatusDescription);
                        return;
                    }

                    if (result.MessageType == WebSocketMessageType.Binary)
                    {
                        memoryStream.Write(buffer, 0, result.Count);
                    }
                    else
                    {
                        _logger.LogWarning("Received non-binary WebSocket message, ignoring");
                    }

                } while (!result.EndOfMessage);

                // Parse complete protobuf message
                var messageBytes = memoryStream.ToArray();
                _metrics.RecordMessageReceived(messageBytes.Length);

                IpcServerMessage message;
                try
                {
                    message = IpcServerMessage.Parser.ParseFrom(messageBytes);
                }
                catch (InvalidProtocolBufferException ex)
                {
                    _logger.LogError(ex, "Failed to parse protobuf message ({ByteCount} bytes)", messageBytes.Length);
                    continue; // Don't crash loop for one bad message
                }

                // Validate message
                if (message.RequestId == 0)
                {
                    _logger.LogWarning("Received message with RequestId=0, ignoring");
                    continue;
                }

                if (message.PayloadCase == IpcServerMessage.PayloadOneofCase.None)
                {
                    _logger.LogWarning("Received message {RequestId} with no payload", message.RequestId);
                    continue;
                }

                _logger.LogDebug("Received response: RequestId={RequestId}, PayloadType={PayloadType}",
                    message.RequestId, message.PayloadCase);

                // Complete pending request
                if (_pendingRequests.TryRemove(message.RequestId, out var tcs))
                {
                    tcs.SetResult(message);
                }
                else
                {
                    _logger.LogWarning(
                        "Received response for unknown RequestId={RequestId}, PayloadType={PayloadType}",
                        message.RequestId, message.PayloadCase);
                }
            }
            catch (OperationCanceledException) when (ct.IsCancellationRequested)
            {
                // Normal shutdown
                break;
            }
            catch (WebSocketException ex)
            {
                _logger.LogError(ex, "WebSocket error in receive loop");

                // Connection lost - fail all pending requests
                var connectionEx = new IpcConnectionException("WebSocket connection lost", ex);
                FailAllPendingRequests(connectionEx);

                SetConnectionState(ConnectionState.Failed);
                break;
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Unexpected error in receive loop");
            }
        }

        _logger.LogDebug("Receive loop exiting");

        // Clean shutdown - fail remaining requests
        FailAllPendingRequests(new IpcConnectionException("Connection closed"));
    }

    /// <summary>
    /// Fails all pending requests with the given exception.
    /// </summary>
    private void FailAllPendingRequests(Exception exception)
    {
        var count = _pendingRequests.Count;
        if (count > 0)
        {
            _logger.LogWarning("Failing {Count} pending requests due to: {Error}", count, exception.Message);
        }

        foreach (var kvp in _pendingRequests)
        {
            if (_pendingRequests.TryRemove(kvp.Key, out var tcs))
            {
                tcs.SetException(exception);
            }
        }
    }

    /// <summary>
    /// Sends a request and waits for response with timeout and cancellation support.
    /// </summary>
    private async Task<IpcServerMessage> SendRequestAsync(
        IpcClientMessage request,
        ulong? requestId = null,
        TimeSpan? timeout = null,
        CancellationToken cancellationToken = default)
    {
        // Check connection state
        if (_ws?.State != WebSocketState.Open)
        {
            throw new IpcConnectionException("Not connected to IPC server",
                endpoint: _ws != null ? $"ws://127.0.0.1" : null,
                wsState: _ws?.State);
        }

        // Generate request ID if not provided (auth uses requestId: 1)
        if (requestId == null)
        {
            requestId = GetNextRequestId();
            request.RequestId = requestId.Value;
        }

        var operationType = request.PayloadCase.ToString();
        timeout ??= _options.DefaultRequestTimeout;

        // Register pending request with RunContinuationsAsynchronously to avoid deadlocks
        var tcs = new TaskCompletionSource<IpcServerMessage>(TaskCreationOptions.RunContinuationsAsynchronously);

        if (!_pendingRequests.TryAdd(requestId.Value, tcs))
        {
            throw new InvalidOperationException($"Duplicate request ID: {requestId}");
        }

        var sw = Stopwatch.StartNew();
        var success = false;

        try
        {
            // Encode protobuf
            byte[] bytes;
            try
            {
                bytes = request.ToByteArray();
            }
            catch (Exception ex)
            {
                throw new IpcProtocolException("Failed to encode request", ex);
            }

            _logger.LogDebug("Sending request: RequestId={RequestId}, Operation={Operation}, Size={Size} bytes",
                requestId, operationType, bytes.Length);

            _metrics.RecordRequestSent(operationType);

            // Send with lock to prevent interleaved sends
            await _sendLock.WaitAsync(cancellationToken);
            try
            {
                await _ws.SendAsync(
                    new ArraySegment<byte>(bytes),
                    WebSocketMessageType.Binary,
                    endOfMessage: true,
                    cancellationToken);
            }
            finally
            {
                _sendLock.Release();
            }

            // Wait for response with timeout and cancellation
            using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            linkedCts.CancelAfter(timeout.Value);

            try
            {
                var responseTask = tcs.Task;
                await responseTask.WaitAsync(linkedCts.Token);

                var response = await responseTask;

                // Check for error response
                if (response.PayloadCase == IpcServerMessage.PayloadOneofCase.Error)
                {
                    throw new IpcServerException(
                        requestId.Value,
                        response.Error.Code,
                        response.Error.Message);
                }

                success = true;
                return response;
            }
            catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
            {
                // Timeout (not user cancellation)
                throw new IpcTimeoutException(requestId.Value, timeout.Value, operationType);
            }
            catch (OperationCanceledException)
            {
                // User cancelled
                _logger.LogDebug("Request {RequestId} cancelled by user", requestId);
                throw;
            }
        }
        finally
        {
            sw.Stop();
            _metrics.RecordRequestCompleted(operationType, sw.Elapsed, success);
            _pendingRequests.TryRemove(requestId.Value, out _);
        }
    }

    // ========== Public API ==========

    public async Task<bool> HealthCheckAsync()
    {
        try
        {
            var request = new IpcClientMessage
            {
                CheckHealth = new IpcCheckHealthRequest()
            };

            using var healthCts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            var response = await SendRequestAsync(request, cancellationToken: healthCts.Token);

            return response.CheckHealthResponse.Healthy;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Health check failed");
            return false;
        }
    }

    public async Task<OcSessionList> ListSessionsAsync(CancellationToken cancellationToken = default)
    {
        var request = new IpcClientMessage
        {
            ListSessions = new IpcListSessionsRequest()
        };

        var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
        return response.SessionList;
    }

    public async Task<OcSessionInfo> CreateSessionAsync(string? title = null,
        CancellationToken cancellationToken = default)
    {
        var request = new IpcClientMessage
        {
            CreateSession = new IpcCreateSessionRequest { Title = title ?? string.Empty }
        };

        var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
        return response.SessionInfo;
    }

    public async Task<bool> DeleteSessionAsync(string sessionId, CancellationToken cancellationToken = default)
    {
        var request = new IpcClientMessage
        {
            DeleteSession = new IpcDeleteSessionRequest { SessionId = sessionId }
        };

        var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
        return response.DeleteSessionResponse.Success;
    }

    // ========== Disposal ==========

    public void Dispose()
    {
        DisposeAsync().AsTask().GetAwaiter().GetResult();
    }

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.CompareExchange(ref _disposed, 1, 0) == 1)
        {
            return; // Already disposed
        }

        _logger.LogInformation("Disposing IPC client");
        SetConnectionState(ConnectionState.Disconnecting);

        // Cancel receive loop
        _cts?.Cancel();

        // Wait for receive task to complete (with timeout)
        if (_receiveTask != null)
        {
            try
            {
                await Task.WhenAny(_receiveTask, Task.Delay(_options.ShutdownTimeout));
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Error waiting for receive task during disposal");
            }
        }

        // Close WebSocket gracefully
        if (_ws?.State == WebSocketState.Open)
        {
            try
            {
                using var closeCts = new CancellationTokenSource(TimeSpan.FromSeconds(2));
                await _ws.CloseAsync(WebSocketCloseStatus.NormalClosure, "Client disposing", closeCts.Token);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Error closing WebSocket during disposal");
            }
        }

        // Dispose resources
        _ws?.Dispose();
        _cts?.Dispose();
        _connectionLock.Dispose();
        _sendLock.Dispose();

        SetConnectionState(ConnectionState.Disconnected);
        _logger.LogInformation("IPC client disposed");
    }

    // ========== Server Management Operations ==========

    public async Task<IpcServerInfo?> DiscoverServerAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        _logger.LogDebug("Discovering OpenCode servers...");

        try
        {
            var request = new IpcClientMessage
            {
                DiscoverServer = new IpcDiscoverServerRequest()
            };

            var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

            // Null-safe: DiscoverServerResponse might be null
            if (response.DiscoverServerResponse == null)
            {
                _logger.LogError("DiscoverServerResponse is null in response payload");
                throw new ServerDiscoveryException("Invalid response from server: DiscoverServerResponse is null");
            }

            var server = response.DiscoverServerResponse.Server;

            if (server != null)
            {
                _logger.LogInformation(
                    "Discovered OpenCode server: {BaseUrl}, PID={Pid}, Owned={Owned}",
                    server.BaseUrl, server.Pid, server.Owned);
            }
            else
            {
                _logger.LogInformation("No OpenCode server found");
            }

            return server;
        }
        catch (IpcException)
        {
            // Re-throw IPC exceptions as-is
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to discover server");
            throw new ServerDiscoveryException("Server discovery failed unexpectedly", ex);
        }
    }

    public async Task<IpcServerInfo> SpawnServerAsync(uint? port = null, CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        var effectivePort = port ?? 0;
        _logger.LogInformation("Spawning OpenCode server on port {Port}...",
            effectivePort == 0 ? "auto" : effectivePort.ToString());

        try
        {
            var request = new IpcClientMessage
            {
                SpawnServer = new IpcSpawnServerRequest { Port = effectivePort }
            };

            var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

            // Validate response
            if (response.SpawnServerResponse == null)
            {
                _logger.LogError("SpawnServerResponse is null in response payload");
                throw new ServerSpawnException("Invalid response from server: SpawnServerResponse is null");
            }

            if (response.SpawnServerResponse.Server == null)
            {
                _logger.LogError("SpawnServerResponse.Server is null");
                throw new ServerSpawnException("Server spawn succeeded but returned null server info");
            }

            var server = response.SpawnServerResponse.Server;

            _logger.LogInformation(
                "OpenCode server spawned successfully: {BaseUrl}, PID={Pid}",
                server.BaseUrl, server.Pid);

            return server;
        }
        catch (IpcException)
        {
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to spawn server");
            throw new ServerSpawnException("Server spawn failed unexpectedly", ex);
        }
    }

    public async Task<bool> StopServerAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        _logger.LogInformation("Stopping OpenCode server...");

        try
        {
            var request = new IpcClientMessage
            {
                StopServer = new IpcStopServerRequest()
            };

            var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

            // Validate response
            if (response.StopServerResponse == null)
            {
                _logger.LogError("StopServerResponse is null in response payload");
                throw new ServerStopException("Invalid response from server: StopServerResponse is null");
            }

            var success = response.StopServerResponse.Success;

            if (success)
            {
                _logger.LogInformation("OpenCode server stopped successfully");
            }
            else
            {
                _logger.LogWarning("Failed to stop server (may not be owned by this client)");
            }

            return success;
        }
        catch (IpcException)
        {
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to stop server");
            throw new ServerStopException("Server stop operation failed unexpectedly", ex);
        }
    }

    public async Task<bool> CheckServerHealthAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        _logger.LogDebug("Checking OpenCode server health...");

        try
        {
            var request = new IpcClientMessage
            {
                CheckHealth = new IpcCheckHealthRequest()
            };

            var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

            // Validate response
            if (response.CheckHealthResponse == null)
            {
                _logger.LogError("CheckHealthResponse is null in response payload");
                throw new ServerHealthCheckException("Invalid response from server: CheckHealthResponse is null");
            }

            var healthy = response.CheckHealthResponse.Healthy;

            _logger.LogDebug("OpenCode server health check: {Status}", healthy ? "healthy" : "unhealthy");

            return healthy;
        }
        catch (IpcException)
        {
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Health check failed");
            throw new ServerHealthCheckException("Server health check failed unexpectedly", ex);
        }
    }

    // ========== Config Operations ==========

    public async Task<(AppConfig App, ModelsConfig Models)> GetConfigAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        _logger.LogDebug("Getting config...");

        try
        {
            var request = new IpcClientMessage
            {
                GetConfig = new IpcGetConfigRequest()
            };

            var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

            // Validate response
            if (response.GetConfigResponse == null)
            {
                _logger.LogError("GetConfigResponse is null in response payload");
                throw new IpcProtocolException("Invalid response from server: GetConfigResponse is null");
            }

            // Deserialize JSON strings to typed objects
            var appConfig = JsonSerializer.Deserialize<AppConfig>(
                                response.GetConfigResponse.AppConfigJson,
                                s_jsonOptions)
                            ?? throw new IpcProtocolException("Failed to deserialize AppConfig");

            var modelsConfig = JsonSerializer.Deserialize<ModelsConfig>(
                                   response.GetConfigResponse.ModelsConfigJson,
                                   s_jsonOptions)
                               ?? throw new IpcProtocolException("Failed to deserialize ModelsConfig");

            _logger.LogInformation(
                "Config loaded: {ModelCount} curated models, default={DefaultModel}",
                modelsConfig.Models.Curated.Count,
                modelsConfig.Models.DefaultModel);

            return (appConfig, modelsConfig);
        }
        catch (IpcException)
        {
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to get config");
            throw new IpcProtocolException("Config retrieval failed unexpectedly", ex);
        }
    }
    
    // Helper method
    private void ThrowIfDisposed()
    {
        if (_disposed == 1)
        {
            throw new ObjectDisposedException(nameof(IpcClient));
        }
    }
}