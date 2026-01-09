namespace OpenCode.Services;

/// <summary>
/// Retry policy for operations that may fail transiently.
/// </summary>
public interface IRetryPolicy
{
  /// <summary>
  /// Executes an operation with retry logic.
  /// </summary>
  /// <typeparam name="T">Return type.</typeparam>
  /// <param name="operation">The async operation to execute.</param>
  /// <param name="shouldRetry">Predicate to determine if exception is retryable. Null = retry all.</param>
  /// <param name="cancellationToken">Cancellation token.</param>
  /// <returns>Result of the operation.</returns>
  /// <exception cref="Exception">Throws the last exception if all retries fail.</exception>
  Task<T> ExecuteAsync<T>(
      Func<CancellationToken, Task<T>> operation,
      Func<Exception, bool>? shouldRetry = null,
      CancellationToken cancellationToken = default);
}
