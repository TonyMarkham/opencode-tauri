using Opencode.Message;

namespace OpenCode.Models;

/// <summary>
/// Wrapper around OcMessage that tracks client-side state.
/// Enables optimistic UI, retry logic, and status tracking.
/// </summary>
public sealed class ChatMessage
{
    /// <summary>Client-generated unique ID (survives retries).</summary>
    public string LocalId { get; }

    /// <summary>The underlying proto message (null while initially sending).</summary>
    public OcMessage? Message { get; set; }

    /// <summary>Current status in the send/receive lifecycle.</summary>
    public MessageStatus Status { get; set; }

    /// <summary>Error message if Status is Failed.</summary>
    public string? ErrorMessage { get; set; }

    /// <summary>Original text (for retry).</summary>
    public string OriginalText { get; }

    /// <summary>Model used (for retry).</summary>
    public string ModelId { get; }

    /// <summary>Provider used (for retry).</summary>
    public string ProviderId { get; }

    /// <summary>When the message was created locally.</summary>
    public DateTime CreatedAt { get; }

    /// <summary>Number of send attempts.</summary>
    public int Attempts { get; set; }

    /// <summary>Whether this is a user message (vs assistant).</summary>
    public bool IsUser => Message?.User!= null
                          || (Message?.Assistant == null && !string.IsNullOrEmpty(OriginalText));

    /// <summary>Whether retry is available.</summary>
    public bool CanRetry => Status == MessageStatus.Failed && Attempts < 3;

    // Constructor for NEW user messages
    public ChatMessage(string text, string modelId, string providerId)
    {
        LocalId = $"local_{Guid.NewGuid():N}";
        OriginalText = text;
        ModelId = modelId;
        ProviderId = providerId;
        Status = MessageStatus.Sending;
        CreatedAt = DateTime.UtcNow;
        Attempts = 1;
    }

    /// <summary>Create from received assistant message.</summary>
    public static ChatMessage FromAssistantMessage(OcMessage message)
    {
        return new ChatMessage(message)
        {
            Status = MessageStatus.Sent
        };
    }

    // Private constructor for assistant messages
    private ChatMessage(OcMessage message)
    {
        LocalId = message.Assistant?.Id 
            ?? message.User?.Id 
            ?? $"local_{Guid.NewGuid():N}";
        Message = message;
        OriginalText = message.Assistant?.Text 
            ?? message.User?.Text ?? "";
        ModelId = message.Assistant?.Model?.ModelId 
            ?? message.User?.Model?.ModelId ?? "";
        ProviderId = message.Assistant?.Model?.ProviderId 
            ?? message.User?.Model?.ProviderId ?? "";
        CreatedAt = DateTime.UtcNow;
        Attempts = 0;
    }
}
