namespace OpenCode.Services;

using Microsoft.JSInterop;
using Microsoft.Extensions.Logging;
using System.Text.Json.Serialization;

/// <summary>
/// Configuration for IPC WebSocket connection.
/// </summary>
public interface IIpcConfigService
{
    /// <summary>
    /// Gets IPC configuration from Tauri backend.
    /// </summary>
    /// <returns>Tuple of (port, authToken)</returns>
    Task<(int Port, string AuthToken)> GetConfigAsync();
}

/// <summary>
/// Retrieves IPC configuration by invoking Tauri command with validation.
/// </summary>
public class TauriIpcConfigService : IIpcConfigService
{
    private readonly IJSRuntime _jsRuntime;
    private readonly ILogger<TauriIpcConfigService> _logger;
    
    public TauriIpcConfigService(IJSRuntime jsRuntime, ILogger<TauriIpcConfigService> logger)
    {
        _jsRuntime = jsRuntime;
        _logger = logger;
    }
    
    public async Task<(int Port, string AuthToken)> GetConfigAsync()
    {
        try
        {
            var result = await _jsRuntime.InvokeAsync<IpcConfigResponse>(
                "window.__TAURI__.core.invoke",
                "get_ipc_config"
            );
            
            // Validate config
            if (result.Port < 1024 || result.Port > 65535)
            {
                throw new InvalidOperationException($"Invalid IPC port: {result.Port}. Must be between 1024-65535.");
            }
            
            if (string.IsNullOrWhiteSpace(result.AuthToken))
            {
                throw new InvalidOperationException("IPC auth token is empty");
            }
            
            if (result.AuthToken.Length < 16)
            {
                _logger.LogWarning("IPC auth token is suspiciously short ({Length} chars)", result.AuthToken.Length);
            }
            
            _logger.LogInformation("IPC config retrieved: port={Port}, token_length={TokenLength}", 
                result.Port, result.AuthToken.Length);
            
            return (result.Port, result.AuthToken);
        }
        catch (JSException ex)
        {
            _logger.LogError(ex, "Failed to invoke Tauri command: get_ipc_config");
            throw new InvalidOperationException("Failed to get IPC configuration from Tauri backend", ex);
        }
    }
    
    // Response type from Rust IpcConfigResponse
    private class IpcConfigResponse
    {
        [JsonPropertyName("port")]
        public int Port { get; set; }
        
        [JsonPropertyName("auth_token")]
        public string AuthToken { get; set; } = string.Empty;
    }
}