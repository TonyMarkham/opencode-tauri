namespace OpenCode.Services;

using System.Diagnostics.Metrics;

/// <summary>
/// Telemetry and metrics for IPC client operations.
/// </summary>
public interface IIpcClientMetrics
{
    void RecordRequestSent(string operationType);
    void RecordRequestCompleted(string operationType, TimeSpan duration, bool success);
    void RecordConnectionStateChange(ConnectionState oldState, ConnectionState newState);
    void RecordMessageReceived(int messageSize);
}

/// <summary>
/// Implementation using System.Diagnostics.Metrics (.NET 6+).
/// </summary>
public class IpcClientMetrics : IIpcClientMetrics
{
    private static readonly Meter s_meter = new("OpenCode.IpcClient", "1.0.0");
    
    private readonly Counter<long> _requestsSent;
    private readonly Histogram<double> _requestDuration;
    private readonly Counter<long> _requestsFailed;
    private readonly Counter<long> _messagesReceived;
    private readonly Histogram<int> _messageSize;
    
    public IpcClientMetrics()
    {
        _requestsSent = s_meter.CreateCounter<long>(
            "ipc.requests.sent", 
            "requests", 
            "Number of IPC requests sent");
            
        _requestDuration = s_meter.CreateHistogram<double>(
            "ipc.request.duration", 
            "ms", 
            "Request duration in milliseconds");
            
        _requestsFailed = s_meter.CreateCounter<long>(
            "ipc.requests.failed", 
            "requests", 
            "Number of failed requests");
            
        _messagesReceived = s_meter.CreateCounter<long>(
            "ipc.messages.received", 
            "messages", 
            "Number of messages received");
            
        _messageSize = s_meter.CreateHistogram<int>(
            "ipc.message.size", 
            "bytes", 
            "Message size in bytes");
    }
    
    public void RecordRequestSent(string operationType)
    {
        _requestsSent.Add(1, new KeyValuePair<string, object?>("operation", operationType));
    }
    
    public void RecordRequestCompleted(string operationType, TimeSpan duration, bool success)
    {
        _requestDuration.Record(duration.TotalMilliseconds, 
            new KeyValuePair<string, object?>("operation", operationType),
            new KeyValuePair<string, object?>("success", success));
        
        if (!success)
        {
            _requestsFailed.Add(1, new KeyValuePair<string, object?>("operation", operationType));
        }
    }
    
    public void RecordMessageReceived(int messageSize)
    {
        _messagesReceived.Add(1);
        _messageSize.Record(messageSize);
    }
    
    public void RecordConnectionStateChange(ConnectionState oldState, ConnectionState newState)
    {
        // Could emit state transition events for monitoring dashboards
        // For now, just a hook for future expansion
    }
}