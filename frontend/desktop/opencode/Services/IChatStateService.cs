namespace OpenCode.Services;

/// <summary>
/// Manages chat state persistence (drafts, preferences).
/// Uses localStorage for cross-session recovery.
/// </summary>
public interface IChatStateService
{
    /// <summary>Save draft message for recovery.</summary>
    ValueTask SaveDraftAsync(string? sessionId, string text);

    /// <summary>Load saved draft message.</summary>
    ValueTask<string?> LoadDraftAsync(string? sessionId);

    /// <summary>Clear draft after successful send.</summary>
    ValueTask ClearDraftAsync(string? sessionId);

    /// <summary>Save last session ID for recovery.</summary>
    ValueTask SaveLastSessionIdAsync(string sessionId);

    /// <summary>Load last session ID.</summary>
    ValueTask<string?> LoadLastSessionIdAsync();
}