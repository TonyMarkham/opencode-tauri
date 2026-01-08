namespace OpenCode.Services;

/// <summary>
/// IPC connection state.
/// </summary>
public enum ConnectionState
{
    /// <summary>Not connected.</summary>
    Disconnected,
    
    /// <summary>WebSocket connection in progress.</summary>
    Connecting,
    
    /// <summary>Performing authentication handshake.</summary>
    Authenticating,
    
    /// <summary>Connected and authenticated.</summary>
    Connected,
    
    /// <summary>Graceful disconnect in progress.</summary>
    Disconnecting,
    
    /// <summary>Connection failed (terminal state).</summary>
    Failed
}

/// <summary>
/// Event args for connection state changes.
/// </summary>
public class ConnectionStateChangedEventArgs : EventArgs
{
    public ConnectionState OldState { get; }
    public ConnectionState NewState { get; }
    
    public ConnectionStateChangedEventArgs(ConnectionState oldState, ConnectionState newState)
    {
        OldState = oldState;
        NewState = newState;
    }
}