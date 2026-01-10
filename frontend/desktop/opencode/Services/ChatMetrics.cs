  using System.Diagnostics;
  using System.Diagnostics.Metrics;

  namespace OpenCode.Services;

  /// <summary>
  /// Production telemetry with correlation ID support.
  /// </summary>
  public sealed class ChatMetrics : IChatMetrics
  {
      private readonly Counter<long> _sessionsCreated;
      private readonly Counter<long> _sessionCreationsFailed;
      private readonly Histogram<double> _sessionCreationDuration;
      private readonly Counter<long> _reconnectionAttempts;
      private readonly Counter<long> _reconnectionSuccesses;
      private readonly Counter<long> _reconnectionFailures;
      private readonly Counter<long> _reconnectionCircuitBreaks;
      private readonly Counter<long> _inputValidationFailures;
      private readonly Counter<long> _inputSanitizations;
      private readonly Counter<long> _draftsSaved;
      private readonly Counter<long> _draftsRestored;
      private readonly Histogram<double> _renderDuration;

      private static readonly ActivitySource ActivitySource = new("OpenCode.Chat");

      public ChatMetrics(IMeterFactory meterFactory)
      {
          var meter = meterFactory.Create("OpenCode.Chat");

          _sessionsCreated = meter.CreateCounter<long>(
              "chat.sessions.created",
              unit: "{session}",
              description: "Chat sessions created successfully");

          _sessionCreationsFailed = meter.CreateCounter<long>(
              "chat.sessions.creation_failed",
              unit: "{error}",
              description: "Failed session creation attempts");

          _sessionCreationDuration = meter.CreateHistogram<double>(
              "chat.sessions.creation_duration_ms",
              unit: "ms",
              description: "Time to create a chat session");

          _reconnectionAttempts = meter.CreateCounter<long>(
              "chat.reconnection.attempts",
              unit: "{attempt}",
              description: "Reconnection attempts made");

          _reconnectionSuccesses = meter.CreateCounter<long>(
              "chat.reconnection.successes",
              unit: "{success}",
              description: "Successful reconnections");

          _reconnectionFailures = meter.CreateCounter<long>(
              "chat.reconnection.failures",
              unit: "{failure}",
              description: "Failed reconnection sequences");

          _reconnectionCircuitBreaks = meter.CreateCounter<long>(
              "chat.reconnection.circuit_breaks",
              unit: "{break}",
              description: "Times circuit breaker prevented reconnection");

          _inputValidationFailures = meter.CreateCounter<long>(
              "chat.input.validation_failed",
              unit: "{failure}",
              description: "Input validation failures");

          _inputSanitizations = meter.CreateCounter<long>(
              "chat.input.sanitized",
              unit: "{sanitization}",
              description: "Inputs that required sanitization");

          _draftsSaved = meter.CreateCounter<long>(
              "chat.drafts.saved",
              unit: "{draft}",
              description: "Draft messages saved");

          _draftsRestored = meter.CreateCounter<long>(
              "chat.drafts.restored",
              unit: "{draft}",
              description: "Draft messages restored");

          _renderDuration = meter.CreateHistogram<double>(
              "chat.render.duration_ms",
              unit: "ms",
              description: "Component render duration");
      }

      public void RecordSessionCreated(string sessionId, string correlationId)
      {
          using var activity = ActivitySource.StartActivity("SessionCreated");
          activity?.SetTag("session_id", sessionId);
          activity?.SetTag("correlation_id", correlationId);

          _sessionsCreated.Add(1,
              new KeyValuePair<string, object?>("session_id", sessionId),
              new KeyValuePair<string, object?>("correlation_id", correlationId));
      }

      public void RecordSessionCreationFailed(string errorType, string correlationId)
      {
          using var activity = ActivitySource.StartActivity("SessionCreationFailed");
          activity?.SetTag("error_type", errorType);
          activity?.SetTag("correlation_id", correlationId);
          activity?.SetStatus(ActivityStatusCode.Error);

          _sessionCreationsFailed.Add(1,
              new KeyValuePair<string, object?>("error_type", errorType),
              new KeyValuePair<string, object?>("correlation_id", correlationId));
      }

      public void RecordSessionCreationDuration(TimeSpan duration, string correlationId)
          => _sessionCreationDuration.Record(duration.TotalMilliseconds,
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordReconnectionAttempt(int attemptNumber, string correlationId)
          => _reconnectionAttempts.Add(1,
              new KeyValuePair<string, object?>("attempt", attemptNumber),
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordReconnectionSuccess(int totalAttempts, TimeSpan totalDuration, string correlationId)
      {
          using var activity = ActivitySource.StartActivity("ReconnectionSuccess");
          activity?.SetTag("total_attempts", totalAttempts);
          activity?.SetTag("total_duration_ms", totalDuration.TotalMilliseconds);

          _reconnectionSuccesses.Add(1,
              new KeyValuePair<string, object?>("total_attempts", totalAttempts),
              new KeyValuePair<string, object?>("correlation_id", correlationId));
      }

      public void RecordReconnectionFailed(int totalAttempts, string correlationId)
      {
          using var activity = ActivitySource.StartActivity("ReconnectionFailed");
          activity?.SetTag("total_attempts", totalAttempts);
          activity?.SetStatus(ActivityStatusCode.Error);

          _reconnectionFailures.Add(1,
              new KeyValuePair<string, object?>("total_attempts", totalAttempts),
              new KeyValuePair<string, object?>("correlation_id", correlationId));
      }

      public void RecordReconnectionCircuitBroken(string correlationId)
          => _reconnectionCircuitBreaks.Add(1,
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordInputValidationFailed(string reason, string correlationId)
          => _inputValidationFailures.Add(1,
              new KeyValuePair<string, object?>("reason", reason),
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordInputSanitized(string correlationId)
          => _inputSanitizations.Add(1,
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordDraftSaved(string correlationId)
          => _draftsSaved.Add(1,
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordDraftRestored(string correlationId)
          => _draftsRestored.Add(1,
              new KeyValuePair<string, object?>("correlation_id", correlationId));

      public void RecordRenderDuration(string component, TimeSpan duration)
          => _renderDuration.Record(duration.TotalMilliseconds,
              new KeyValuePair<string, object?>("component", component));

      public string GenerateCorrelationId()
          => Activity.Current?.Id ?? Guid.NewGuid().ToString("N")[..16];
  }