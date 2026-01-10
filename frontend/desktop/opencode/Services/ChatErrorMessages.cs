  namespace OpenCode.Services;

  /// <summary>
  /// Centralized, i18n-ready error messages.
  /// All user-facing strings in one place for easy localization.
  /// </summary>
  public static class ChatErrorMessages
  {
      // Session Creation
      public const string SessionCreationFailed = "Failed to create chat session. Please try again.";
      public const string SessionCreationTimeout = "Session creation timed out. The server may be busy.";

      // Connection
      public const string ConnectionLost = "Connection to backend lost.";
      public const string ConnectionFailed = "Unable to connect to backend.";
      public const string ReconnectionFailed = "Failed to reconnect after {0} attempts. Click Retry to try again.";
      public const string Reconnecting = "Connection lost. Reconnecting... (Attempt {0} of {1})";
      public const string ReconnectionCooldown = "Please wait before attempting to reconnect.";

      // Input Validation
      public const string InputTooLong = "Message is too long. Maximum {0:N0} characters allowed.";
      public const string InputEmpty = "Please enter a message.";
      public const string InputContainsDangerousContent = "Message contains invalid content that was removed.";

      // Generic
      public const string UnexpectedError = "An unexpected error occurred. Please try again.";
      public const string ServerError = "Server error: {0}";
      public const string OfflineMode = "You appear to be offline. Messages will be sent when connection is restored.";

      // Titles (for alert headers)
      public const string TitleConnectionError = "Connection Error";
      public const string TitleSessionError = "Session Error";
      public const string TitleTimeout = "Request Timeout";
      public const string TitleServerError = "Server Error";
      public const string TitleValidationError = "Validation Error";
      public const string TitleOffline = "Offline";

      // Accessibility announcements
      public const string A11ySessionCreated = "Chat session created successfully";
      public const string A11yReconnecting = "Attempting to reconnect";
      public const string A11yReconnected = "Connection restored";
      public const string A11yMessageSent = "Message sent";
      public const string A11yError = "Error: {0}";

      // Placeholders
      public const string InputPlaceholder = "Type a message... (Ctrl+Enter to send)";
      public const string EmptyStateTitle = "Start a conversation";
      public const string EmptyStateSubtitle = "Type a message below to begin";

      /// <summary>Format a message with parameters.</summary>
      public static string Format(string template, params object[] args)
          => string.Format(template, args);
  }