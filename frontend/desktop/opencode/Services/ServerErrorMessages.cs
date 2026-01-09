namespace OpenCode.Services;

/// <summary>
/// Localized error messages for server operations.
/// Future: Replace with resource files for i18n.
/// </summary>
public static class ServerErrorMessages
{
    // Discovery errors
    public const string DiscoveryFailed = "Failed to discover OpenCode server. Please check your connection.";
    public const string DiscoveryTimeout = "Server discovery timed out. The operation took too long.";
    
    // Spawn errors
    public const string SpawnFailed = "Failed to start OpenCode server. Please check the logs.";
    public const string SpawnTimeout = "Server startup timed out. Please try again.";
    public const string SpawnPortInUse = "Failed to start server: Port is already in use.";
    
    // Stop errors
    public const string StopFailed = "Failed to stop OpenCode server.";
    public const string StopNotOwned = "Cannot stop server: This server was not started by this application.";
    public const string StopTimeout = "Server stop operation timed out.";
    
    // Health check errors
    public const string HealthCheckFailed = "Server health check failed. The server may be unresponsive.";
    public const string HealthCheckTimeout = "Health check timed out.";
    
    // Connection errors
    public const string IpcDisconnected = "Not connected to IPC server. Please reconnect.";
    public const string UnexpectedError = "An unexpected error occurred. Please try again.";
}