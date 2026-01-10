namespace OpenCode.Services;

/// <summary>
/// Comprehensive metrics interface for chat operations.
/// Supports correlation IDs for distributed tracing.
/// </summary>
public interface IChatMetrics
{
    // Session metrics
    void RecordSessionCreated(string sessionId, string correlationId);
    void RecordSessionCreationFailed(string errorType, string correlationId);
    void RecordSessionCreationDuration(TimeSpan duration, string correlationId);

    // Reconnection metrics
    void RecordReconnectionAttempt(int attemptNumber, string correlationId);
    void RecordReconnectionSuccess(int totalAttempts, TimeSpan totalDuration, string correlationId);
    void RecordReconnectionFailed(int totalAttempts, string correlationId);
    void RecordReconnectionCircuitBroken(string correlationId);

    // Input metrics
    void RecordInputValidationFailed(string reason, string correlationId);
    void RecordInputSanitized(string correlationId);
    void RecordDraftSaved(string correlationId);
    void RecordDraftRestored(string correlationId);

    // Performance metrics
    void RecordRenderDuration(string component, TimeSpan duration);

    /// <summary>Generate a new correlation ID for an operation.</summary>
    string GenerateCorrelationId();
}