namespace OpenCode.Services;

/// <summary>
/// Configuration options for chat functionality.
/// Supports the Options pattern for DI and testing.
/// </summary>
public interface IChatOptions
{
    /// <summary>Maximum message length in characters.</summary>
    int MaxMessageLength { get; }

    /// <summary>Maximum reconnection attempts before giving up.</summary>
    int MaxReconnectAttempts { get; }

    /// <summary>Initial delay between reconnection attempts.</summary>
    TimeSpan InitialReconnectDelay { get; }

    /// <summary>Maximum delay between reconnection attempts (backoff cap).</summary>
    TimeSpan MaxReconnectDelay { get; }

    /// <summary>Minimum time between reconnection sequences (circuit breaker).</summary>
    TimeSpan ReconnectCooldown { get; }

    /// <summary>Debounce delay for input validation.</summary>
    TimeSpan ValidationDebounceDelay { get; }

    /// <summary>Whether to persist draft messages to localStorage.</summary>
    bool EnableDraftPersistence { get; }

    /// <summary>Session title for new chat sessions.</summary>
    string DefaultSessionTitle { get; }
}