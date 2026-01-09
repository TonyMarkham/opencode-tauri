namespace OpenCode.Services;

/// <summary>
/// Configuration options for retry policy.
/// </summary>
public class RetryPolicyOptions
{
    /// <summary>Maximum retry attempts (default: 3).</summary>
    public int MaxRetries { get; set; } = 3;

    /// <summary>Initial delay before first retry (default: 100ms).</summary>
    public TimeSpan InitialDelay { get; set; } = TimeSpan.FromMilliseconds(100);

    /// <summary>Maximum delay between retries (default: 2s).</summary>
    public TimeSpan MaxDelay { get; set; } = TimeSpan.FromSeconds(2);

    /// <summary>Multiplier for exponential backoff (default: 2.0).</summary>
    public double BackoffMultiplier { get; set; } = 2.0;

    /// <summary>Add random jitter to prevent thundering herd (default: true).</summary>
    public bool AddJitter { get; set; } = true;
}