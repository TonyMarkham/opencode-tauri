  namespace OpenCode.Services;

  using Microsoft.Extensions.Logging;
  using OpenCode.Services.Exceptions;

  /// <summary>
  /// Centralized configuration service with caching, retry, and connection awareness.
  /// </summary>
  public class ConfigService : IConfigService, IDisposable
  {
      private readonly IIpcClient _ipcClient;
      private readonly IRetryPolicy _retryPolicy;
      private readonly ILogger<ConfigService> _logger;
      private readonly SemaphoreSlim _loadLock = new(1, 1);
      private readonly TimeSpan _defaultMaxAge = TimeSpan.FromSeconds(30);

      // Cached state
      private AppConfig? _appConfig;
      private ModelsConfig? _modelsConfig;
      private ConfigLoadState _state = ConfigLoadState.NotLoaded;
      private string? _errorMessage;
      private DateTime? _lastLoadedAt;

      // Request deduplication
      private Task<(AppConfig?, ModelsConfig?)>? _inFlightLoad;

      public AppConfig? AppConfig => _appConfig;
      public ModelsConfig? ModelsConfig => _modelsConfig;
      public ConfigLoadState State => _state;
      public string? ErrorMessage => _errorMessage;
      public DateTime? LastLoadedAt => _lastLoadedAt;

      public event EventHandler<ConfigChangedEventArgs>? ConfigChanged;

      public ConfigService(
          IIpcClient ipcClient,
          IRetryPolicy retryPolicy,
          ILogger<ConfigService> logger)
      {
          _ipcClient = ipcClient;
          _retryPolicy = retryPolicy;
          _logger = logger;

          // Subscribe to connection state changes
          _ipcClient.ConnectionStateChanged += OnConnectionStateChanged;
      }

      public async Task<(AppConfig? App, ModelsConfig? Models)> GetConfigAsync(
          TimeSpan? maxAge = null,
          CancellationToken cancellationToken = default)
      {
          maxAge ??= _defaultMaxAge;

          // If we have fresh cached data, return immediately
          if (_state == ConfigLoadState.Loaded && _lastLoadedAt != null)
          {
              var age = DateTime.UtcNow - _lastLoadedAt.Value;
              if (age < maxAge)
              {
                  _logger.LogDebug("Returning cached config (age: {AgeMs}ms)", age.TotalMilliseconds);
                  return (_appConfig, _modelsConfig);
              }

              // Data is stale - trigger background refresh but return cached data
              _logger.LogDebug("Config is stale (age: {AgeMs}ms), triggering background refresh", age.TotalMilliseconds);
              _ = Task.Run(() => RefreshAsync(CancellationToken.None)); // Fire and forget
              return (_appConfig, _modelsConfig);
          }

          // No cached data or in error state - load now
          await RefreshAsync(cancellationToken);
          return (_appConfig, _modelsConfig);
      }

      public async Task RefreshAsync(CancellationToken cancellationToken = default)
      {
          // Request deduplication: if load is already in progress, await it
          await _loadLock.WaitAsync(cancellationToken);
          try
          {
              if (_inFlightLoad != null)
              {
                  _logger.LogDebug("Config load already in progress, awaiting existing request");
                  await _inFlightLoad;
                  return;
              }

              // Start new load
              _inFlightLoad = LoadConfigInternalAsync(cancellationToken);
              await _inFlightLoad;
          }
          finally
          {
              _inFlightLoad = null;
              _loadLock.Release();
          }
      }

      private async Task<(AppConfig?, ModelsConfig?)> LoadConfigInternalAsync(CancellationToken cancellationToken)
      {
          // Check if connected
          if (!_ipcClient.IsConnected)
          {
              _logger.LogWarning("Cannot load config: IPC not connected");
              SetState(
                  _modelsConfig != null ? ConfigLoadState.Stale : ConfigLoadState.Error,
                  "Not connected to IPC server");
              return (_appConfig, _modelsConfig);
          }

          SetState(ConfigLoadState.Loading, null);

          try
          {
              // Use retry policy for transient failures
              var result = await _retryPolicy.ExecuteAsync(
                  async ct => await _ipcClient.GetConfigAsync(ct),
                  IsRetryableException,
                  cancellationToken);

              // Success
              _appConfig = result.App;
              _modelsConfig = result.Models;
              _lastLoadedAt = DateTime.UtcNow;

              _logger.LogInformation(
                  "Config loaded successfully: {ModelCount} models, default={DefaultModel}",
                  _modelsConfig?.Models.Curated.Count ?? 0,
                  _modelsConfig?.Models.DefaultModel ?? "none");

              SetState(ConfigLoadState.Loaded, null);
              return (_appConfig, _modelsConfig);
          }
          catch (OperationCanceledException)
          {
              _logger.LogDebug("Config load cancelled");
              throw;
          }
          catch (Exception ex)
          {
              _logger.LogError(ex, "Config load failed after retries");

              // Graceful degradation: if we have cached data, mark as stale
              var newState = _modelsConfig != null ? ConfigLoadState.Stale : ConfigLoadState.Error;
              SetState(newState, ex.Message);

              return (_appConfig, _modelsConfig);
          }
      }

      private void SetState(ConfigLoadState newState, string? errorMessage)
      {
          if (_state != newState || _errorMessage != errorMessage)
          {
              _state = newState;
              _errorMessage = errorMessage;

              _logger.LogDebug("Config state changed: {State}", newState);
              ConfigChanged?.Invoke(this, new ConfigChangedEventArgs(newState, errorMessage));
          }
      }

      private void OnConnectionStateChanged(object? sender, ConnectionStateChangedEventArgs e)
      {
          // When reconnected, trigger a refresh if we're in error state
          if (e.NewState == ConnectionState.Connected && _state == ConfigLoadState.Error)
          {
              _logger.LogInformation("IPC reconnected, refreshing config");
              _ = Task.Run(() => RefreshAsync(CancellationToken.None));
          }
      }

      private static bool IsRetryableException(Exception ex) => ex switch
      {
          IpcTimeoutException => true,
          IpcConnectionException => false, // Don't retry - wait for reconnection event
          _ => false
      };

      public void Dispose()
      {
          _ipcClient.ConnectionStateChanged -= OnConnectionStateChanged;
          _loadLock.Dispose();
      }
  }