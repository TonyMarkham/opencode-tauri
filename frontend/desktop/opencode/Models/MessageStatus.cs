namespace OpenCode.Models;

/// <summary>
/// Tracks the lifecycle state of a chat message.
/// </summary>
public enum MessageStatus
{
    /// <summary>User message being sent to server.</summary>
    Sending,

    /// <summary>Message successfully sent/received.</summary>
    Sent,

    /// <summary>Message failed to send (retryable).</summary>
    Failed,

    /// <summary>Waiting for assistant response.</summary>
    AwaitingResponse
}