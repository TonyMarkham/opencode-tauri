# Authentication (`auth.proto`)

**Status:** ✅ Complete  
**Last Updated:** 2026-01-05

---

## Purpose

Track OAuth vs API key mode per provider, provider connections, OAuth expiry. Auth is **per-provider**, not global — each provider (Anthropic, OpenAI, etc.) has its own authentication configuration.

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/oauth.schema.json` - OAuth authentication with refresh/access tokens
- `submodules/opencode/schema/apiAuth.schema.json` - API key authentication  
- `submodules/opencode/schema/wellKnownAuth.schema.json` - Well-known authentication (key + token pair)
- `submodules/opencode/schema/auth.schema.json` - Discriminated union of all auth types

**Previously derived from (now superseded):**

- `packages/opencode/src/auth/index.ts` @ `c50f588` (2026-01-05) lines 7-33

---

## Messages

```protobuf
syntax = "proto3";
package opencode.auth;

// OAuth authentication credentials
// Source: submodules/opencode/schema/oauth.schema.json (canonical)
message OAuth {
  string type = 1;                     // "oauth" (constant discriminator)
  string refresh = 2;                  // OAuth refresh token
  string access = 3;                   // OAuth access token
  double expires = 4;                  // Token expiration (Unix timestamp ms)
  optional string enterprise_url = 5;  // Optional enterprise-specific OAuth endpoint (JSON: enterpriseUrl)
}

// API key authentication credentials
// Source: submodules/opencode/schema/apiAuth.schema.json (canonical)
message ApiAuth {
  string type = 1;                     // "api" (constant discriminator)
  string key = 2;                      // API key for authentication
}

// Well-known authentication credentials (key + token pair)
// Source: submodules/opencode/schema/wellKnownAuth.schema.json (canonical)
message WellKnownAuth {
  string type = 1;                     // "wellknown" (constant discriminator)
  string key = 2;                      // Well-known authentication key
  string token = 3;                    // Well-known authentication token
}

// Authentication credentials (discriminated union)
// Source: submodules/opencode/schema/auth.schema.json (canonical)
message Auth {
  oneof auth {
    OAuth oauth = 1;
    ApiAuth api = 2;
    WellKnownAuth well_known = 3;     // JSON: wellKnown
  }
}

message Empty {}
```

---

## Service Definition

**Note:** The OpenCode server stores auth in `~/.local/share/opencode/auth.json` as a `Record<string, Auth>` (provider ID → Auth).

For protobuf/gRPC, this could be exposed as:

```protobuf
service AuthService {
  rpc GetAllAuth(Empty) returns (AllAuth);                  // Get all provider auth (map)
  rpc GetAuth(GetAuthRequest) returns (Auth);               // Get single provider auth
  rpc SetAuth(SetAuthRequest) returns (Empty);              // Set provider auth
  rpc RemoveAuth(RemoveAuthRequest) returns (Empty);        // Remove provider auth
}

message AllAuth {
  map<string, Auth> auth = 1;  // Provider ID → Auth credentials
}

message GetAuthRequest {
  string provider_id = 1;
}

message SetAuthRequest {
  string provider_id = 1;
  Auth auth = 2;
}

message RemoveAuthRequest {
  string provider_id = 1;
}
```

---

## Maps to OpenCode Server

The OpenCode server exposes auth through TypeScript functions in `src/auth/index.ts`:

- `Auth.all()` → Returns `Record<string, Auth.Info>` from `auth.json`
- `Auth.get(providerID)` → Returns auth for single provider
- `Auth.set(key, info)` → Writes auth for provider to `auth.json`
- `Auth.remove(key)` → Removes provider auth from `auth.json`

File location: `~/.local/share/opencode/auth.json`

---

## Design Notes

**Why auth is per-provider:**

- Different providers support different auth methods (API key, OAuth, well-known)
- OAuth tokens have per-provider expiry times  
- Users may have some providers configured via env vars, others via OAuth
- Each provider's auth is stored independently in `auth.json`

**Auth types (discriminated union):**

- **OAuth** - Full OAuth flow with refresh/access tokens and expiry
- **ApiAuth** - Simple API key authentication
- **WellKnownAuth** - Key + token pair for well-known credentials

**Discriminated union in protobuf:**

The JSON Schema uses `oneOf` with a `type` discriminator field. In protobuf, this maps to a `oneof` with separate message types for each variant. The `type` field in each variant is a constant string literal (`z.literal("oauth")` → `"oauth"` constant).

---

## JSON Schema Cross-Reference

### OAuth Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `type` | `type` | const "oauth" | Discriminator |
| `refresh` | `refresh` | string | OAuth refresh token |
| `access` | `access` | string | OAuth access token |
| `expires` | `expires` | number | Unix timestamp ms |
| `enterprise_url` | `enterpriseUrl` | string (optional) | Naming: snake_case vs camelCase |

### ApiAuth Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `type` | `type` | const "api" | Discriminator |
| `key` | `key` | string | API key |

### WellKnownAuth Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `type` | `type` | const "wellknown" | Discriminator |
| `key` | `key` | string | Well-known key |
| `token` | `token` | string | Well-known token |

### Auth Message (Union)

| Protobuf | JSON Schema | Notes |
|----------|-------------|-------|
| `oneof auth` with 3 variants | `oneOf` with 3 schemas | Discriminated union on `type` field |
| `OAuth oauth` | `$ref: oauth.schema.json` | OAuth variant |
| `ApiAuth api` | `$ref: apiAuth.schema.json` | API key variant |
| `WellKnownAuth well_known` | `$ref: wellKnownAuth.schema.json` | Well-known variant |
