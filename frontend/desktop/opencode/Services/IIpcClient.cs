namespace OpenCode.Services;

using Opencode;
using Opencode.Session;

/// <summary>
/// IPC client for communication with Rust backend via WebSocket + protobuf.
/// </summary>
public interface IIpcClient : IAsyncDisposable
{
    /// <summary>
    /// Connects to IPC server and performs authentication handshake.
    /// </summary>
    /// <exception cref="Exceptions.IpcConnectionException">WebSocket connection failed.</exception>
    /// <exception cref="Exceptions.IpcAuthenticationException">Authentication failed.</exception>
    Task ConnectAsync();

    /// <summary>
    /// Returns true if currently connected and authenticated.
    /// </summary>
    bool IsConnected { get; }

    /// <summary>
    /// Performs health check on IPC connection.
    /// </summary>
    /// <returns>True if healthy, false otherwise.</returns>
    Task<bool> HealthCheckAsync();

    /// <summary>
    /// Event raised when connection state changes.
    /// </summary>
    event EventHandler<ConnectionStateChangedEventArgs>? ConnectionStateChanged;

    // Session operations

    /// <summary>
    /// Lists all sessions from OpenCode server.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected.</exception>
    /// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
    /// <exception cref="Exceptions.IpcServerException">Server returned error.</exception>
    Task<OcSessionList> ListSessionsAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Creates a new session.
    /// </summary>
    /// <param name="title">Session title (optional).</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected.</exception>
    /// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
    /// <exception cref="Exceptions.IpcServerException">Server returned error.</exception>
    Task<OcSessionInfo> CreateSessionAsync(string? title = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// Deletes a session.
    /// </summary>
    /// <param name="sessionId">Session ID to delete.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>True if deleted, false if not found.</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected.</exception>
    /// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
    /// <exception cref="Exceptions.IpcServerException">Server returned error.</exception>
    Task<bool> DeleteSessionAsync(string sessionId, CancellationToken cancellationToken = default);

    // Server management operations

    /// <summary>
    /// Discovers running OpenCode servers on localhost.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>Server info if found, null if no server running.</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
    /// <exception cref="Exceptions.ServerDiscoveryException">Discovery operation failed.</exception>
    Task<IpcServerInfo?> DiscoverServerAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Spawns a new OpenCode server process.
    /// </summary>
    /// <param name="port">Preferred port (0 = auto-assign).</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>Spawned server info.</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
    /// <exception cref="Exceptions.ServerSpawnException">Server spawn failed.</exception>
    Task<IpcServerInfo> SpawnServerAsync(uint? port = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// Stops the OpenCode server (only works if spawned by this client).
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>True if stopped successfully, false if not owned or already stopped.</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
    /// <exception cref="Exceptions.ServerStopException">Stop operation failed.</exception>
    Task<bool> StopServerAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if the OpenCode server is healthy and responding.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>True if server is responding, false otherwise.</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
    /// <exception cref="Exceptions.ServerHealthCheckException">Health check failed.</exception>
    Task<bool> CheckServerHealthAsync(CancellationToken cancellationToken = default);
    
    // Config operations

    /// <summary>
    /// Gets the current application and models configuration.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>Tuple of (AppConfig, ModelsConfig).</returns>
    /// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
    /// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
    Task<(AppConfig App, ModelsConfig Models)> GetConfigAsync(CancellationToken cancellationToken = default);
}