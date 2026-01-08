namespace OpenCode.Services;

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
}