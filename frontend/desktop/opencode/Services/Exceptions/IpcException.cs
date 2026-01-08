namespace OpenCode.Services.Exceptions;

/// <summary>
/// Base exception for IPC operation failures.
/// </summary>
public abstract class IpcException : Exception
{
    protected IpcException(string message) : base(message) { }
    protected IpcException(string message, Exception innerException) 
        : base(message, innerException) { }
}

/// <summary>
/// Exception thrown when WebSocket connection fails.
/// </summary>
public class IpcConnectionException : IpcException
{
    public string? Endpoint { get; }
    public System.Net.WebSockets.WebSocketState? WebSocketState { get; }
    
    public IpcConnectionException(string message, string? endpoint = null, System.Net.WebSockets.WebSocketState? wsState = null) 
        : base(message)
    {
        Endpoint = endpoint;
        WebSocketState = wsState;
    }
    
    public IpcConnectionException(string message, Exception innerException, string? endpoint = null) 
        : base(message, innerException)
    {
        Endpoint = endpoint;
    }
}

/// <summary>
/// Exception thrown when authentication handshake fails.
/// </summary>
public class IpcAuthenticationException : IpcException
{
    public IpcAuthenticationException(string message) : base(message) { }
    public IpcAuthenticationException(string message, Exception innerException) 
        : base(message, innerException) { }
}

/// <summary>
/// Exception thrown when protobuf encoding/decoding fails.
/// </summary>
public class IpcProtocolException : IpcException
{
    public IpcProtocolException(string message) : base(message) { }
    public IpcProtocolException(string message, Exception innerException) 
        : base(message, innerException) { }
}

/// <summary>
/// Exception thrown when request times out waiting for response.
/// </summary>
public class IpcTimeoutException : IpcException
{
    public ulong RequestId { get; }
    public TimeSpan Timeout { get; }
    public string? OperationType { get; }
    
    public IpcTimeoutException(ulong requestId, TimeSpan timeout, string? operationType = null)
        : base($"Request {requestId} ({operationType ?? "unknown"}) timed out after {timeout.TotalSeconds}s")
    {
        RequestId = requestId;
        Timeout = timeout;
        OperationType = operationType;
    }
}

/// <summary>
/// Exception thrown when server returns an error response.
/// </summary>
public class IpcServerException : IpcException
{
    public Opencode.IpcErrorCode ErrorCode { get; }
    public ulong RequestId { get; }
    
    public IpcServerException(ulong requestId, Opencode.IpcErrorCode code, string message) 
        : base($"Server error ({code}): {message}")
    {
        RequestId = requestId;
        ErrorCode = code;
    }
}