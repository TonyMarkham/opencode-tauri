  namespace OpenCode.Services;

  using System.Diagnostics;
  using Microsoft.Extensions.Logging;
  using Opencode;

  /// <summary>
  /// Manages auth sync operations with proper exclusivity and cancellation.
  ///
  /// Design:
  /// - SemaphoreSlim(1,1) ensures only one operation at a time
  /// - No race conditions from rapid button clicks
  /// - Proper cancellation propagation
  /// - Metrics with error categorization
  /// </summary>
  public interface IAuthSyncService
  {
      /// <summary>
      /// Whether a sync operation is currently in progress.
      /// </summary>
      bool IsOperationInProgress { get; }

      /// <summary>
      /// Sync API keys from .env to server.
      /// </summary>
      /// <param name="skipOAuthProviders">Skip providers with OAuth configured.</param>
      /// <param name="cancellationToken">Cancellation token.</param>
      /// <returns>Sync result.</returns>
      Task<AuthSyncResult> SyncKeysAsync(
          bool skipOAuthProviders = true,
          CancellationToken cancellationToken = default);

      /// <summary>
      /// Cancel any in-progress operation.
      /// </summary>
      void CancelCurrentOperation();
  }

  public class AuthSyncService : IAuthSyncService, IDisposable
  {
      private readonly IIpcClient _ipcClient;
      private readonly ILogger<AuthSyncService> _logger;

      // Ensures only one operation at a time - no race conditions
      private readonly SemaphoreSlim _operationLock = new(1, 1);

      // Current operation's cancellation source
      private CancellationTokenSource? _currentCts;
      private readonly object _ctsLock = new();

      public AuthSyncService(
          IIpcClient ipcClient,
          ILogger<AuthSyncService> logger)
      {
          _ipcClient = ipcClient ?? throw new ArgumentNullException(nameof(ipcClient));
          _logger = logger ?? throw new ArgumentNullException(nameof(logger));
      }

      public bool IsOperationInProgress => _operationLock.CurrentCount == 0;

      public async Task<AuthSyncResult> SyncKeysAsync(
          bool skipOAuthProviders = true,
          CancellationToken cancellationToken = default)
      {
          // Try to acquire lock with timeout
          var acquired = await _operationLock.WaitAsync(TimeSpan.FromMilliseconds(100), cancellationToken);
          if (!acquired)
          {
              _logger.LogWarning("Sync operation already in progress, rejecting new request");
              throw new InvalidOperationException("A sync operation is already in progress");
          }

          // Create linked cancellation source
          CancellationTokenSource linkedCts;
          lock (_ctsLock)
          {
              _currentCts?.Dispose();
              _currentCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
              linkedCts = _currentCts;
          }

          var stopwatch = Stopwatch.StartNew();

          try
          {
              _logger.LogInformation("Starting auth sync (skipOAuth={SkipOAuth})", skipOAuthProviders);

              // Ensure connected
              if (!_ipcClient.IsConnected)
              {
                  _logger.LogDebug("IPC not connected, connecting...");
                  await _ipcClient.ConnectAsync(linkedCts.Token);
              }

              // Call IPC
              var response = await _ipcClient.SyncAuthKeysAsync(
                  skipOAuthProviders,
                  linkedCts.Token);

              stopwatch.Stop();

              _logger.LogInformation(
                  "Auth sync completed in {Duration}ms: {Synced} synced, {Failed} failed, {Skipped} skipped, {Invalid} invalid",
                  stopwatch.ElapsedMilliseconds,
                  response.Synced.Count,
                  response.Failed.Count,
                  response.Skipped.Count,
                  response.ValidationFailed.Count);

              return new AuthSyncResult
              {
                  Response = response,
                  Duration = stopwatch.Elapsed,
                  Success = true
              };
          }
          catch (OperationCanceledException)
          {
              _logger.LogInformation("Auth sync cancelled after {Duration}ms", stopwatch.ElapsedMilliseconds);

              return new AuthSyncResult
              {
                  Response = null,
                  Duration = stopwatch.Elapsed,
                  Success = false,
                  Error = "Operation was cancelled"
              };
          }
          catch (Exception ex)
          {
              _logger.LogError(ex, "Auth sync failed after {Duration}ms", stopwatch.ElapsedMilliseconds);

              return new AuthSyncResult
              {
                  Response = null,
                  Duration = stopwatch.Elapsed,
                  Success = false,
                  Error = ex.Message
              };
          }
          finally
          {
              _operationLock.Release();

              lock (_ctsLock)
              {
                  if (_currentCts == linkedCts)
                  {
                      _currentCts = null;
                  }
              }
              linkedCts.Dispose();
          }
      }

      public void CancelCurrentOperation()
      {
          lock (_ctsLock)
          {
              if (_currentCts is { IsCancellationRequested: false })
              {
                  _logger.LogDebug("Cancelling current auth sync operation");
                  _currentCts.Cancel();
              }
          }
      }

      public void Dispose()
      {
          CancelCurrentOperation();
          _operationLock.Dispose();

          lock (_ctsLock)
          {
              _currentCts?.Dispose();
              _currentCts = null;
          }
      }
  }

  /// <summary>
  /// Result of an auth sync operation.
  /// </summary>
  public class AuthSyncResult
  {
      public IpcAuthSyncResponse? Response { get; init; }
      public TimeSpan Duration { get; init; }
      public bool Success { get; init; }
      public string? Error { get; init; }
  }