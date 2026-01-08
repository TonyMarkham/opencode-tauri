namespace OpenCode.Services;

/// <summary>
/// Configuration options for IPC client.
/// </summary>
public class IpcClientOptions
{
    /// <summary>
    /// Default timeout for IPC requests.
    /// </summary>
    public TimeSpan DefaultRequestTimeout { get; set; } = TimeSpan.FromSeconds(30);
    
    /// <summary>
    /// Timeout for WebSocket connection.
    /// </summary>
    public TimeSpan ConnectionTimeout { get; set; } = TimeSpan.FromSeconds(10);
    
    /// <summary>
    /// Timeout for authentication handshake.
    /// </summary>
    public TimeSpan AuthenticationTimeout { get; set; } = TimeSpan.FromSeconds(5);
    
    /// <summary>
    /// Timeout for graceful shutdown.
    /// </summary>
    public TimeSpan ShutdownTimeout { get; set; } = TimeSpan.FromSeconds(5);
    
    /// <summary>
    /// Maximum receive buffer size (bytes).
    /// </summary>
    public int MaxReceiveBufferSize { get; set; } = 64 * 1024; // 64KB
}