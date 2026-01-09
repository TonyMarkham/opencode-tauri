namespace OpenCode.Services;

using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;

/// <summary>
/// Implementation of retry policy with exponential backoff and jitter.
/// </summary>
public class RetryPolicy : IRetryPolicy
{
  private readonly RetryPolicyOptions _options;
  private readonly ILogger<RetryPolicy> _logger;
  private static readonly Random s_random = new();

  public RetryPolicy(IOptions<RetryPolicyOptions> options, ILogger<RetryPolicy> logger)
  {
      _options = options.Value;
      _logger = logger;
  }

  public async Task<T> ExecuteAsync<T>(
      Func<CancellationToken, Task<T>> operation,
      Func<Exception, bool>? shouldRetry = null,
      CancellationToken cancellationToken = default)
  {
      ArgumentNullException.ThrowIfNull(operation);

      var attempt = 0;
      Exception? lastException = null;

      while (attempt <= _options.MaxRetries)
      {
          try
          {
              return await operation(cancellationToken);
          }
          catch (OperationCanceledException)
          {
              // Don't retry cancellations
              throw;
          }
          catch (Exception ex)
          {
              lastException = ex;

              // Check if we should retry this exception
              var isRetryable = shouldRetry?.Invoke(ex) ?? true;
              if (!isRetryable)
              {
                  _logger.LogDebug(ex, "Exception is not retryable, failing immediately");
                  throw;
              }

              // Check if we have attempts left
              if (attempt >= _options.MaxRetries)
              {
                  _logger.LogWarning(ex, "All {MaxRetries} retry attempts exhausted", _options.MaxRetries);
                  throw;
              }

              // Calculate delay with exponential backoff
              var delay = CalculateDelay(attempt);

              _logger.LogWarning(
                  ex,
                  "Operation failed (attempt {Attempt}/{MaxAttempts}), retrying in {DelayMs}ms",
                  attempt + 1,
                  _options.MaxRetries + 1,
                  delay.TotalMilliseconds);

              await Task.Delay(delay, cancellationToken);
              attempt++;
          }
      }

      // Should never reach here, but just in case
      throw lastException ?? new InvalidOperationException("Retry loop exited unexpectedly");
  }

  private TimeSpan CalculateDelay(int attempt)
  {
      // Exponential backoff: InitialDelay * (BackoffMultiplier ^ attempt)
      var exponentialDelay = _options.InitialDelay.TotalMilliseconds
          * Math.Pow(_options.BackoffMultiplier, attempt);

      // Cap at MaxDelay
      var cappedDelay = Math.Min(exponentialDelay, _options.MaxDelay.TotalMilliseconds);

      // Add jitter: ±25% randomization
      if (_options.AddJitter)
      {
          lock (s_random)
          {
              var jitterFactor = 0.75 + (s_random.NextDouble() * 0.5); // 0.75 to 1.25
              cappedDelay *= jitterFactor;
          }
      }

      return TimeSpan.FromMilliseconds(cappedDelay);
  }
}