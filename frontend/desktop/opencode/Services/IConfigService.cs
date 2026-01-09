namespace OpenCode.Services;

/// <summary>
/// Centralized configuration management with caching and state tracking.
/// </summary>
public interface IConfigService
{
    /// <summary>Current models config (null if never loaded successfully).</summary>
    ModelsConfig? ModelsConfig { get; }

    /// <summary>Current app config (null if never loaded successfully).</summary>
    AppConfig? AppConfig { get; }

    /// <summary>Current load state.</summary>
    ConfigLoadState State { get; }

    /// <summary>Error message if State == Error or Stale.</summary>
    string? ErrorMessage { get; }

    /// <summary>When config was last successfully loaded (UTC).</summary>
    DateTime? LastLoadedAt { get; }

    /// <summary>
    /// Gets config, loading if necessary. Returns cached data immediately if fresh.
    /// Triggers background refresh if data is stale.
    /// </summary>
    /// <param name="maxAge">Maximum age before triggering background refresh. Default 30s.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>Tuple of (AppConfig, ModelsConfig), may be null if never loaded and load fails.</returns>
    Task<(AppConfig? App, ModelsConfig? Models)> GetConfigAsync(
        TimeSpan? maxAge = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// Forces a refresh, returns when complete.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    Task RefreshAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Raised when config changes or state changes.
    /// </summary>
    event EventHandler<ConfigChangedEventArgs>? ConfigChanged;
    
    /// <summary>
    /// Gets display name for a provider ID.
    /// </summary>
    /// <param name="providerId">Provider ID (e.g., "openai").</param>
    /// <returns>Display name (e.g., "OpenAI") or provider ID if not found.</returns>
    string GetProviderDisplayName(string providerId);
}