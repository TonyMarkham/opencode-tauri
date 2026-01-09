# Session 12: Auth Sync - Production Implementation Plan

**Goal:** API keys sync to OpenCode server on connect
**Demo:** See "Synced: openai, anthropic" in settings
**Estimated slices:** 6 implementation slices

---

## Overview

This session implements automatic syncing of API keys from a `.env` file to the OpenCode server. When the desktop client connects to an OpenCode server, it reads `*_API_KEY` environment variables and PUTs them to the server's `/auth/{provider}` endpoint.

### Data Flow

```
.env file → load_env_api_keys() → OpencodeClient.sync_auth() → PUT /auth/{provider}
                                                                     ↓
                                              AuthSection.razor ← IpcAuthSyncStatusResponse
```

---

## Slice 1: Proto Updates

**File:** `proto/ipc.proto`

### 1.1 Add Request Message (line ~52, in IpcClientMessage payload)

```protobuf
// Auth Sync (52)
IpcSyncAuthKeysRequest sync_auth_keys = 52;
```

### 1.2 Add Response Message (line ~89, in IpcServerMessage payload)

```protobuf
// Auth Sync Status (51)
IpcAuthSyncStatusResponse auth_sync_status = 51;
```

### 1.3 Add Message Definitions (after line 199)

```protobuf
// ============================================
// AUTH SYNC OPERATIONS
// ============================================

// Request to sync API keys from .env to OpenCode server
message IpcSyncAuthKeysRequest {}

// Response with sync results per provider
message IpcAuthSyncStatusResponse {
  repeated string synced_providers = 1;           // Successfully synced (e.g., ["openai", "anthropic"])
  repeated IpcAuthSyncFailure failed_providers = 2;  // Failed syncs with error details
}

// Individual provider sync failure
message IpcAuthSyncFailure {
  string provider = 1;   // Provider ID (e.g., "openai")
  string error = 2;      // Error message
}
```

### 1.4 Verification

```bash
cd backend/client-core && cargo build
```

Proto compilation should succeed and generate new Rust types.

---

## Slice 2: Backend - Auth Sync Module

**New file:** `backend/client-core/src/auth_sync.rs`

### 2.1 Module Structure

```rust
//! API key synchronization from .env to OpenCode server.
//!
//! This module handles:
//! - Loading .env file from executable directory
//! - Extracting *_API_KEY environment variables
//! - Provider name normalization (OPENAI_API_KEY → openai)

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use log::{debug, info, warn};

/// Loads API keys from .env file and environment.
///
/// Searches for .env in:
/// 1. Current working directory
/// 2. Executable directory
///
/// Returns HashMap of provider → key (e.g., "openai" → "sk-...")
///
/// # Security
/// - Never logs actual key values
/// - Skips empty and placeholder values
pub fn load_env_api_keys() -> HashMap<String, String> {
    // Try to load .env file (non-fatal if missing)
    if let Err(e) = try_load_dotenv() {
        debug!("No .env file loaded: {}", e);
    }

    let mut keys = HashMap::new();

    for (var_name, value) in env::vars() {
        if let Some(provider) = extract_provider_name(&var_name) {
            if is_valid_api_key(&value) {
                info!("Found API key for provider: {}", provider);
                keys.insert(provider, value);
            }
        }
    }

    keys
}

/// Attempts to load .env from known locations.
fn try_load_dotenv() -> Result<PathBuf, String> {
    // Try current directory first
    if let Ok(path) = dotenvy::dotenv() {
        return Ok(path);
    }

    // Try executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            if env_path.exists() {
                dotenvy::from_path(&env_path)
                    .map_err(|e| e.to_string())?;
                return Ok(env_path);
            }
        }
    }

    Err("No .env file found".to_string())
}

/// Extracts provider name from environment variable.
///
/// Examples:
/// - "OPENAI_API_KEY" → Some("openai")
/// - "ANTHROPIC_API_KEY" → Some("anthropic")
/// - "PATH" → None
fn extract_provider_name(env_var: &str) -> Option<String> {
    if env_var.ends_with("_API_KEY") {
        let provider = env_var.strip_suffix("_API_KEY")?;
        Some(provider.to_lowercase())
    } else {
        None
    }
}

/// Validates that a key value is not empty or a placeholder.
fn is_valid_api_key(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.contains("your-api-key")
        && !value.contains("sk-xxx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_provider_name_openai() {
        assert_eq!(extract_provider_name("OPENAI_API_KEY"), Some("openai".to_string()));
    }

    #[test]
    fn extract_provider_name_anthropic() {
        assert_eq!(extract_provider_name("ANTHROPIC_API_KEY"), Some("anthropic".to_string()));
    }

    #[test]
    fn extract_provider_name_google() {
        assert_eq!(extract_provider_name("GOOGLE_API_KEY"), Some("google".to_string()));
    }

    #[test]
    fn extract_provider_name_non_api_key() {
        assert_eq!(extract_provider_name("PATH"), None);
        assert_eq!(extract_provider_name("HOME"), None);
    }

    #[test]
    fn is_valid_api_key_real_key() {
        assert!(is_valid_api_key("sk-proj-abc123"));
    }

    #[test]
    fn is_valid_api_key_placeholder() {
        assert!(!is_valid_api_key(""));
        assert!(!is_valid_api_key("sk-..."));
        assert!(!is_valid_api_key("your-api-key-here"));
    }
}
```

### 2.2 Add Dependency

**File:** `backend/client-core/Cargo.toml`

```toml
dotenvy = "0.15"
```

### 2.3 Register Module

**File:** `backend/client-core/src/lib.rs`

Add:
```rust
pub mod auth_sync;
```

---

## Slice 3: Backend - OpencodeClient Auth Method

**File:** `backend/client-core/src/opencode_client/mod.rs`

### 3.1 Add Constant

```rust
const OPENCODE_SERVER_AUTH_ENDPOINT: &str = "auth";
```

### 3.2 Add Method (after `delete_session`)

```rust
/// Syncs an API key to the OpenCode server for a provider.
///
/// Calls PUT /auth/{provider} with body: {"type": "api", "key": "..."}
///
/// # Arguments
/// - `provider`: Provider ID (e.g., "openai", "anthropic")
/// - `api_key`: The API key to sync
///
/// # Security
/// - API key is only transmitted, never logged
pub async fn sync_auth(
    &self,
    provider: &str,
    api_key: &str,
) -> Result<(), OpencodeClientError> {
    let url = self
        .base_url
        .join(&format!("{OPENCODE_SERVER_AUTH_ENDPOINT}/{provider}"))?;

    let body = serde_json::json!({
        "type": "api",
        "key": api_key,
    });

    let response = self
        .prepare_request(self.client.put(url))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OpencodeClientError::Server {
            message: format!(
                "HTTP {} - {}",
                response.status().as_u16(),
                response.text().await.unwrap_or_default()
            ),
            location: ErrorLocation::from(Location::caller()),
        });
    }

    Ok(())
}
```

---

## Slice 4: Backend - IPC Handler

**File:** `backend/client-core/src/ipc/server.rs`

### 4.1 Add Import

```rust
use crate::auth_sync::load_env_api_keys;
use crate::proto::{IpcSyncAuthKeysRequest, IpcAuthSyncStatusResponse, IpcAuthSyncFailure};
```

### 4.2 Add Handler (after `handle_update_config`)

```rust
/// Handles sync_auth_keys request - loads .env and syncs to OpenCode server.
async fn handle_sync_auth_keys(
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling sync_auth_keys request");

    // Get OpenCode client (must be connected to a server)
    let client = state
        .get_opencode_client()
        .await
        .ok_or_else(|| IpcError::Io {
            message: "No OpenCode server connected. Discover or spawn a server first.".to_string(),
            location: ErrorLocation::from(Location::caller()),
        })?;

    // Load API keys from .env
    let keys = load_env_api_keys();

    if keys.is_empty() {
        info!("No API keys found in environment");
        let response = IpcServerMessage {
            request_id,
            payload: Some(ipc_server_message::Payload::AuthSyncStatus(
                IpcAuthSyncStatusResponse {
                    synced_providers: vec![],
                    failed_providers: vec![],
                },
            )),
        };
        return send_protobuf_response(write, &response).await;
    }

    // Sync each key
    let mut synced = Vec::new();
    let mut failed = Vec::new();

    for (provider, key) in keys {
        info!("Syncing auth for provider: {}", provider);

        match client.sync_auth(&provider, &key).await {
            Ok(_) => {
                info!("Successfully synced auth for: {}", provider);
                synced.push(provider);
            }
            Err(e) => {
                warn!("Failed to sync auth for {}: {}", provider, e);
                failed.push(IpcAuthSyncFailure {
                    provider,
                    error: e.to_string(),
                });
            }
        }
    }

    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::AuthSyncStatus(
            IpcAuthSyncStatusResponse {
                synced_providers: synced,
                failed_providers: failed,
            },
        )),
    };

    send_protobuf_response(write, &response).await
}
```

### 4.3 Add Match Arm (in `dispatch_message` match block, after config handlers)

```rust
Payload::SyncAuthKeys(_req) => {
    handle_sync_auth_keys(state, request_id, write).await
}
```

### 4.4 Verification

```bash
cd backend/client-core && cargo build && cargo test
```

---

## Slice 5: Frontend - IpcClient Updates

### 5.1 Create AuthSyncStatus DTO

**New file:** `frontend/desktop/opencode/Services/AuthSyncStatus.cs`

```csharp
namespace OpenCode.Services;

/// <summary>
/// Result of auth key synchronization operation.
/// </summary>
public class AuthSyncStatus
{
    /// <summary>
    /// Providers that were successfully synced (e.g., ["openai", "anthropic"]).
    /// </summary>
    public List<string> SyncedProviders { get; init; } = [];

    /// <summary>
    /// Providers that failed to sync, with error messages.
    /// </summary>
    public Dictionary<string, string> FailedProviders { get; init; } = [];

    /// <summary>
    /// True if any providers were synced successfully.
    /// </summary>
    public bool HasSyncedAny => SyncedProviders.Count > 0;

    /// <summary>
    /// True if any providers failed to sync.
    /// </summary>
    public bool HasFailedAny => FailedProviders.Count > 0;

    /// <summary>
    /// True if no keys were found to sync.
    /// </summary>
    public bool NoKeysFound => SyncedProviders.Count == 0 && FailedProviders.Count == 0;

    /// <summary>
    /// Human-readable summary of sync status.
    /// </summary>
    public string Summary => (SyncedProviders.Count, FailedProviders.Count) switch
    {
        (0, 0) => "No API keys found in .env",
        (> 0, 0) => $"Synced: {string.Join(", ", SyncedProviders)}",
        (0, > 0) => $"Failed: {string.Join(", ", FailedProviders.Keys)}",
        (> 0, > 0) => $"Synced: {string.Join(", ", SyncedProviders)} | Failed: {string.Join(", ", FailedProviders.Keys)}",
        _ => "Unknown status"
    };
}
```

### 5.2 Add Interface Method

**File:** `frontend/desktop/opencode/Services/IIpcClient.cs`

Add after `GetConfigAsync`:

```csharp
// Auth sync operations

/// <summary>
/// Syncs API keys from .env file to OpenCode server.
/// </summary>
/// <param name="cancellationToken">Cancellation token.</param>
/// <returns>Sync status with results per provider.</returns>
/// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
/// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
/// <exception cref="Exceptions.AuthSyncException">Sync operation failed.</exception>
Task<AuthSyncStatus> SyncAuthKeysAsync(CancellationToken cancellationToken = default);
```

### 5.3 Add Exception Type

**New file:** `frontend/desktop/opencode/Services/Exceptions/AuthSyncException.cs`

```csharp
namespace OpenCode.Services.Exceptions;

/// <summary>
/// Thrown when auth key synchronization fails.
/// </summary>
public class AuthSyncException : IpcException
{
    public AuthSyncException(string message) : base(message) { }
    public AuthSyncException(string message, Exception inner) : base(message, inner) { }
}
```

### 5.4 Implement Method

**File:** `frontend/desktop/opencode/Services/IpcClient.cs`

Add implementation (follow existing patterns like `ListSessionsAsync`):

```csharp
public async Task<AuthSyncStatus> SyncAuthKeysAsync(CancellationToken cancellationToken = default)
{
    EnsureConnected();

    var request = new IpcClientMessage
    {
        RequestId = GetNextRequestId(),
        SyncAuthKeys = new IpcSyncAuthKeysRequest()
    };

    Logger.LogDebug("Sending sync_auth_keys request (id={RequestId})", request.RequestId);

    var response = await SendRequestAsync(request, cancellationToken);

    if (response.AuthSyncStatus != null)
    {
        var status = response.AuthSyncStatus;
        return new AuthSyncStatus
        {
            SyncedProviders = status.SyncedProviders.ToList(),
            FailedProviders = status.FailedProviders
                .ToDictionary(f => f.Provider, f => f.Error)
        };
    }

    if (response.Error != null)
    {
        throw new AuthSyncException($"Server error: {response.Error.Message}");
    }

    throw new AuthSyncException("Unexpected response type for sync_auth_keys");
}
```

---

## Slice 6: Frontend - AuthSection Component

**New file:** `frontend/desktop/opencode/Components/AuthSection.razor`

### 6.1 Component Template

```razor
@namespace OpenCode.Components
@inject IIpcClient IpcClient
@inject ILogger<AuthSection> Logger
@using OpenCode.Services
@using OpenCode.Services.Exceptions
@implements IDisposable

<RadzenFieldset Text="API Keys" Style="margin-bottom: 1rem;" aria-label="API Key Synchronization">
    <RadzenStack Gap="1rem">

        @* Status Row *@
        <RadzenRow AlignItems="AlignItems.Center">
            <RadzenColumn Size="3">
                <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                    Status
                </RadzenText>
            </RadzenColumn>
            <RadzenColumn Size="9">
                <RadzenStack Orientation="Orientation.Horizontal" AlignItems="AlignItems.Center" Gap="0.5rem">
                    <RadzenIcon Icon="@GetStatusIcon()" Style="@GetStatusColor()" aria-hidden="true" />
                    <RadzenText TextStyle="TextStyle.Body1" role="status" aria-live="polite">
                        @GetStatusText()
                    </RadzenText>
                </RadzenStack>
            </RadzenColumn>
        </RadzenRow>

        @* Synced Providers *@
        @if (_syncStatus?.SyncedProviders.Count > 0)
        {
            <RadzenRow AlignItems="AlignItems.Start">
                <RadzenColumn Size="3">
                    <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                        Synced
                    </RadzenText>
                </RadzenColumn>
                <RadzenColumn Size="9">
                    <RadzenStack Orientation="Orientation.Horizontal" Gap="0.25rem" Wrap="FlexWrap.Wrap">
                        @foreach (var provider in _syncStatus.SyncedProviders)
                        {
                            <RadzenBadge BadgeStyle="BadgeStyle.Success" Text="@FormatProviderName(provider)" />
                        }
                    </RadzenStack>
                </RadzenColumn>
            </RadzenRow>
        }

        @* Failed Providers *@
        @if (_syncStatus?.FailedProviders.Count > 0)
        {
            <RadzenAlert
                AlertStyle="AlertStyle.Warning"
                Variant="Variant.Flat"
                Shade="Shade.Lighter"
                AllowClose="false"
                role="alert">
                <strong>Sync failures:</strong>
                <ul style="margin: 0.5rem 0 0 0; padding-left: 1.5rem;">
                    @foreach (var (provider, error) in _syncStatus.FailedProviders)
                    {
                        <li><strong>@FormatProviderName(provider):</strong> @error</li>
                    }
                </ul>
            </RadzenAlert>
        }

        @* Error Display *@
        @if (_error != null)
        {
            <RadzenAlert
                AlertStyle="AlertStyle.Danger"
                Variant="Variant.Flat"
                Shade="Shade.Lighter"
                AllowClose="true"
                Close="@DismissError"
                role="alert"
                aria-live="assertive">
                @_error
            </RadzenAlert>
        }

        @* Action Button *@
        <RadzenStack Orientation="Orientation.Horizontal" Gap="0.5rem" role="group" aria-label="Auth actions">
            <RadzenButton
                Text="Sync Keys"
                Icon="sync"
                ButtonStyle="ButtonStyle.Primary"
                Click="SyncKeysAsync"
                Disabled="_loading"
                aria-label="Sync API keys from .env file"
                title="Load API keys from .env and sync to OpenCode server" />
        </RadzenStack>

        @if (_loading)
        {
            <RadzenProgressBar Mode="ProgressBarMode.Indeterminate" Style="height: 4px;" aria-label="Syncing..." />
        }

        @* Help Text *@
        <RadzenText TextStyle="TextStyle.Caption" Style="color: var(--rz-text-tertiary-color);">
            Reads OPENAI_API_KEY, ANTHROPIC_API_KEY, etc. from .env file
        </RadzenText>

    </RadzenStack>
</RadzenFieldset>

@code {
    private AuthSyncStatus? _syncStatus;
    private bool _loading;
    private string? _error;
    private CancellationTokenSource? _cts;

    private async Task SyncKeysAsync()
    {
        await CancelCurrentOperationAsync();

        _cts = new CancellationTokenSource();
        _loading = true;
        _error = null;

        try
        {
            if (!IpcClient.IsConnected)
            {
                Logger.LogDebug("IPC not connected, connecting...");
                await IpcClient.ConnectAsync();
            }

            _syncStatus = await IpcClient.SyncAuthKeysAsync(_cts.Token);

            Logger.LogInformation("Auth sync completed: {SyncedCount} synced, {FailedCount} failed",
                _syncStatus.SyncedProviders.Count,
                _syncStatus.FailedProviders.Count);
        }
        catch (OperationCanceledException)
        {
            Logger.LogDebug("Auth sync cancelled");
        }
        catch (IpcConnectionException ex)
        {
            _error = "IPC connection failed. Please try again.";
            Logger.LogError(ex, "IPC connection error during auth sync");
        }
        catch (IpcTimeoutException ex)
        {
            _error = "Sync operation timed out. Please try again.";
            Logger.LogError(ex, "Timeout during auth sync");
        }
        catch (AuthSyncException ex)
        {
            _error = $"Sync failed: {ex.Message}";
            Logger.LogError(ex, "Auth sync operation failed");
        }
        catch (Exception ex)
        {
            _error = "Unexpected error during sync.";
            Logger.LogError(ex, "Unexpected error during auth sync");
        }
        finally
        {
            _loading = false;
        }
    }

    private async Task CancelCurrentOperationAsync()
    {
        if (_cts is { IsCancellationRequested: false })
        {
            await _cts.CancelAsync();
            _cts.Dispose();
            _cts = null;
            await Task.Delay(50);
        }
    }

    private string GetStatusIcon() => _syncStatus switch
    {
        null => "hourglass_empty",
        { NoKeysFound: true } => "info",
        { HasSyncedAny: true, HasFailedAny: false } => "check_circle",
        { HasSyncedAny: true, HasFailedAny: true } => "warning",
        { HasSyncedAny: false, HasFailedAny: true } => "error",
        _ => "help"
    };

    private string GetStatusColor() => _syncStatus switch
    {
        null => "color: var(--rz-text-disabled-color);",
        { NoKeysFound: true } => "color: var(--rz-text-secondary-color);",
        { HasSyncedAny: true, HasFailedAny: false } => "color: var(--rz-success);",
        { HasSyncedAny: true, HasFailedAny: true } => "color: var(--rz-warning);",
        { HasSyncedAny: false, HasFailedAny: true } => "color: var(--rz-danger);",
        _ => "color: var(--rz-text-disabled-color);"
    };

    private string GetStatusText() => _syncStatus switch
    {
        null => "Not synced",
        { NoKeysFound: true } => "No API keys found",
        _ => _syncStatus.Summary
    };

    private static string FormatProviderName(string provider) => provider.ToLowerInvariant() switch
    {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "google" => "Google",
        "openrouter" => "OpenRouter",
        "azure" => "Azure",
        _ => provider.ToUpperInvariant()
    };

    private void DismissError() => _error = null;

    public void Dispose()
    {
        _cts?.Cancel();
        _cts?.Dispose();
    }
}
```

### 6.2 Add to SettingsModal

**File:** `frontend/desktop/opencode/Components/SettingsModal.razor`

Add `<AuthSection />` after `<ServerSection />`:

```razor
<ServerSection />
<AuthSection />
<ModelsSection />
```

---

## Verification Checklist

### Build Verification
```bash
# Backend
cd backend/client-core
cargo build
cargo test
cargo clippy

# Frontend
cd frontend/desktop/opencode
dotnet build
```

### Manual Testing

1. **Setup:**
   - Create `.env` file in project root with:
     ```
     OPENAI_API_KEY=sk-test-123
     ANTHROPIC_API_KEY=sk-ant-test-456
     ```

2. **Test Flow:**
   - Start OpenCode server (`cargo run -p opencode-server`)
   - Launch Tauri app (`cargo tauri dev`)
   - Open Settings modal
   - Click "Sync Keys" button
   - Verify: Green badges appear for synced providers

3. **Edge Cases:**
   - No .env file → Shows "No API keys found"
   - Empty .env → Shows "No API keys found"
   - Invalid key (placeholder) → Skipped, not synced
   - Server not connected → Shows error message

---

## Success Criteria

- [ ] "Sync Keys" button triggers auth sync
- [ ] Synced providers show green badges
- [ ] Failed providers show warning with error details
- [ ] No API keys are logged (security)
- [ ] Works without .env file (graceful degradation)
- [ ] Loading state shows progress bar
- [ ] Errors are dismissible

---

## Future Enhancements (Out of Scope)

These are NOT part of Session 12 but noted for future sessions:

1. **Auto-sync on server connect** - Hook into SetServer to trigger sync automatically
2. **Skip Anthropic if OAuth** - Check for OAuth tokens before syncing Anthropic API key
3. **Retry failed syncs** - Individual retry buttons per failed provider
4. **Real-time .env watching** - Detect .env changes and prompt resync

---

## File Manifest

### Files to Create
| File | Purpose |
|------|---------|
| `backend/client-core/src/auth_sync.rs` | .env loading and key extraction |
| `frontend/desktop/opencode/Services/AuthSyncStatus.cs` | Sync result DTO |
| `frontend/desktop/opencode/Services/Exceptions/AuthSyncException.cs` | Exception type |
| `frontend/desktop/opencode/Components/AuthSection.razor` | Settings UI component |

### Files to Modify
| File | Changes |
|------|---------|
| `proto/ipc.proto` | Add sync messages |
| `backend/client-core/Cargo.toml` | Add dotenvy dependency |
| `backend/client-core/src/lib.rs` | Register auth_sync module |
| `backend/client-core/src/opencode_client/mod.rs` | Add sync_auth method |
| `backend/client-core/src/ipc/server.rs` | Add handler + dispatch |
| `frontend/desktop/opencode/Services/IIpcClient.cs` | Add interface method |
| `frontend/desktop/opencode/Services/IpcClient.cs` | Add implementation |
| `frontend/desktop/opencode/Components/SettingsModal.razor` | Add AuthSection |

---

## Reference Patterns

All implementations should follow patterns from:

- **Rust handlers:** `backend/client-core/src/ipc/server.rs` lines 623-731
- **Rust HTTP client:** `backend/client-core/src/opencode_client/mod.rs`
- **C# component:** `frontend/desktop/opencode/Components/ServerSection.razor`
- **C# interface:** `frontend/desktop/opencode/Services/IIpcClient.cs`
- **Reference impl:** `submodules/opencode-egui/src/startup/auth.rs`
