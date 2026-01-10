  using System.ComponentModel.DataAnnotations;

  namespace OpenCode.Services;

  /// <summary>
  /// Default chat options with validation.
  /// Can be configured via appsettings or overridden in tests.
  /// </summary>
  public sealed class ChatOptions : IChatOptions
  {
      public const string SectionName = "Chat";

      [Range(100, 100000)]
      public int MaxMessageLength { get; set; } = 10000;

      [Range(1, 20)]
      public int MaxReconnectAttempts { get; set; } = 5;

      public TimeSpan InitialReconnectDelay { get; set; } = TimeSpan.FromSeconds(1);

      public TimeSpan MaxReconnectDelay { get; set; } = TimeSpan.FromSeconds(30);

      public TimeSpan ReconnectCooldown { get; set; } = TimeSpan.FromSeconds(60);

      public TimeSpan ValidationDebounceDelay { get; set; } = TimeSpan.FromMilliseconds(150);

      public bool EnableDraftPersistence { get; set; } = true;

      public string DefaultSessionTitle { get; set; } = "Chat Session";

      /// <summary>
      /// Validates all options and throws if invalid.
      /// </summary>
      public void Validate()
      {
          var context = new ValidationContext(this);
          var results = new List<ValidationResult>();

          if (!Validator.TryValidateObject(this, context, results, validateAllProperties: true))
          {
              throw new OptionsValidationException(
                  nameof(ChatOptions),
                  typeof(ChatOptions),
                  results.Select(r => r.ErrorMessage ?? "Validation failed"));
          }

          if (MaxReconnectDelay < InitialReconnectDelay)
              throw new OptionsValidationException(
                  nameof(ChatOptions),
                  typeof(ChatOptions),
                  new[] { "MaxReconnectDelay must be >= InitialReconnectDelay" });
      }
  }

  /// <summary>
  /// Exception thrown when options validation fails.
  /// </summary>
  public class OptionsValidationException : Exception
  {
      public string OptionsName { get; }
      public Type OptionsType { get; }
      public IEnumerable<string> Failures { get; }

      public OptionsValidationException(string name, Type type, IEnumerable<string> failures)
          : base($"Options validation failed for '{name}': {string.Join(", ", failures)}")
      {
          OptionsName = name;
          OptionsType = type;
          Failures = failures;
      }
  }