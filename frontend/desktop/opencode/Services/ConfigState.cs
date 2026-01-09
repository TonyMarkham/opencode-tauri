namespace OpenCode.Services;

/// <summary>
/// Represents the current state of config loading.
/// </summary>
public enum ConfigLoadState
{
    /// <summary>Config has never been loaded.</summary>
    NotLoaded,

    /// <summary>Load request is currently in flight.</summary>
    Loading,

    /// <summary>Config is loaded and fresh.</summary>
    Loaded,

    /// <summary>Have cached config, but refresh failed.</summary>
    Stale,

    /// <summary>Load failed and no cached data available.</summary>
    Error
}

/// <summary>
/// Event args for config change notifications.
/// </summary>
public class ConfigChangedEventArgs : EventArgs
{
    public ConfigLoadState State { get; }
    public string? ErrorMessage { get; }

    public ConfigChangedEventArgs(ConfigLoadState state, string? errorMessage = null)
    {
        State = state;
        ErrorMessage = errorMessage;
    }
}