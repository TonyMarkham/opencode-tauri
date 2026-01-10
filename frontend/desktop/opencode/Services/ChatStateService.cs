  using Microsoft.JSInterop;

  namespace OpenCode.Services;

  /// <summary>
  /// localStorage-based state persistence for chat.
  /// Gracefully degrades if localStorage unavailable.
  /// </summary>
  public sealed class ChatStateService : IChatStateService
  {
      private const string DraftKeyPrefix = "opencode.chat.draft.";
      private const string LastSessionKey = "opencode.chat.lastSession";

      private readonly IJSRuntime _js;
      private readonly ILogger<ChatStateService> _logger;
      private readonly IChatOptions _options;
      private bool _isAvailable = true;

      public ChatStateService(IJSRuntime js, ILogger<ChatStateService> logger, IChatOptions options)
      {
          _js = js;
          _logger = logger;
          _options = options;
      }

      public async ValueTask SaveDraftAsync(string? sessionId, string text)
      {
          if (!_options.EnableDraftPersistence || !_isAvailable) return;

          try
          {
              var key = GetDraftKey(sessionId);
              if (string.IsNullOrEmpty(text))
              {
                  await _js.InvokeVoidAsync("localStorage.removeItem", key);
              }
              else
              {
                  await _js.InvokeVoidAsync("localStorage.setItem", key, text);
              }
          }
          catch (Exception ex)
          {
              _logger.LogWarning(ex, "Failed to save draft to localStorage");
              _isAvailable = false; // Stop trying if localStorage fails
          }
      }

      public async ValueTask<string?> LoadDraftAsync(string? sessionId)
      {
          if (!_options.EnableDraftPersistence || !_isAvailable) return null;

          try
          {
              var key = GetDraftKey(sessionId);
              return await _js.InvokeAsync<string?>("localStorage.getItem", key);
          }
          catch (Exception ex)
          {
              _logger.LogWarning(ex, "Failed to load draft from localStorage");
              _isAvailable = false;
              return null;
          }
      }

      public async ValueTask ClearDraftAsync(string? sessionId)
      {
          if (!_options.EnableDraftPersistence || !_isAvailable) return;

          try
          {
              var key = GetDraftKey(sessionId);
              await _js.InvokeVoidAsync("localStorage.removeItem", key);
          }
          catch (Exception ex)
          {
              _logger.LogWarning(ex, "Failed to clear draft from localStorage");
          }
      }

      public async ValueTask SaveLastSessionIdAsync(string sessionId)
      {
          if (!_isAvailable) return;

          try
          {
              await _js.InvokeVoidAsync("localStorage.setItem", LastSessionKey, sessionId);
          }
          catch (Exception ex)
          {
              _logger.LogWarning(ex, "Failed to save last session ID");
          }
      }

      public async ValueTask<string?> LoadLastSessionIdAsync()
      {
          if (!_isAvailable) return null;

          try
          {
              return await _js.InvokeAsync<string?>("localStorage.getItem", LastSessionKey);
          }
          catch (Exception ex)
          {
              _logger.LogWarning(ex, "Failed to load last session ID");
              return null;
          }
      }

      private static string GetDraftKey(string? sessionId)
          => sessionId != null ? $"{DraftKeyPrefix}{sessionId}" : $"{DraftKeyPrefix}new";
  }