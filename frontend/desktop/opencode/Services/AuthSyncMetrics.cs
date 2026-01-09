  namespace OpenCode.Services;

  using System.Diagnostics.Metrics;

  /// <summary>
  /// Telemetry for auth sync operations with error categorization.
  /// </summary>
  public interface IAuthSyncMetrics
  {
      void RecordSyncAttempt();
      void RecordSyncCompleted(int synced, int failed, int skipped, int invalid, TimeSpan duration);
      void RecordSyncCancelled(TimeSpan duration);
      void RecordSyncFailed(TimeSpan duration, string exceptionType);
      void RecordProviderResult(string provider, string result, string? errorCategory);
  }

  public class AuthSyncMetrics : IAuthSyncMetrics
  {
      private static readonly Meter s_meter = new("OpenCode.AuthSync", "1.0.0");

      private readonly Counter<long> _syncAttempts;
      private readonly Counter<long> _syncCompleted;
      private readonly Counter<long> _syncCancelled;
      private readonly Counter<long> _syncFailed;
      private readonly Histogram<double> _syncDuration;
      private readonly Counter<long> _providerResults;

      public AuthSyncMetrics()
      {
          _syncAttempts = s_meter.CreateCounter<long>(
              "auth.sync.attempts",
              "attempts",
              "Number of auth sync attempts");

          _syncCompleted = s_meter.CreateCounter<long>(
              "auth.sync.completed",
              "operations",
              "Number of completed auth syncs");

          _syncCancelled = s_meter.CreateCounter<long>(
              "auth.sync.cancelled",
              "operations",
              "Number of cancelled auth syncs");

          _syncFailed = s_meter.CreateCounter<long>(
              "auth.sync.failed",
              "operations",
              "Number of failed auth syncs");

          _syncDuration = s_meter.CreateHistogram<double>(
              "auth.sync.duration",
              "ms",
              "Auth sync duration in milliseconds");

          _providerResults = s_meter.CreateCounter<long>(
              "auth.provider.results",
              "providers",
              "Per-provider sync results");
      }

      public void RecordSyncAttempt()
      {
          _syncAttempts.Add(1);
      }

      public void RecordSyncCompleted(int synced, int failed, int skipped, int invalid, TimeSpan duration)
      {
          _syncCompleted.Add(1,
              new KeyValuePair<string, object?>("synced_count", synced),
              new KeyValuePair<string, object?>("failed_count", failed),
              new KeyValuePair<string, object?>("skipped_count", skipped),
              new KeyValuePair<string, object?>("invalid_count", invalid));

          _syncDuration.Record(duration.TotalMilliseconds,
              new KeyValuePair<string, object?>("result", "completed"));
      }

      public void RecordSyncCancelled(TimeSpan duration)
      {
          _syncCancelled.Add(1);
          _syncDuration.Record(duration.TotalMilliseconds,
              new KeyValuePair<string, object?>("result", "cancelled"));
      }

      public void RecordSyncFailed(TimeSpan duration, string exceptionType)
      {
          _syncFailed.Add(1,
              new KeyValuePair<string, object?>("exception_type", exceptionType));

          _syncDuration.Record(duration.TotalMilliseconds,
              new KeyValuePair<string, object?>("result", "failed"),
              new KeyValuePair<string, object?>("exception_type", exceptionType));
      }

      public void RecordProviderResult(string provider, string result, string? errorCategory)
      {
          var tags = new List<KeyValuePair<string, object?>>
          {
              new("provider", provider),
              new("result", result)
          };

          if (errorCategory is not null)
          {
              tags.Add(new("error_category", errorCategory));
          }

          _providerResults.Add(1, tags.ToArray());
      }
  }