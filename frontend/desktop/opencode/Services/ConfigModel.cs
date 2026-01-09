namespace OpenCode.Services;

using System.Text.Json.Serialization;

// ============================================
// APP CONFIG (mirrors backend/client-core/src/config/mod.rs)
// ============================================

public record AppConfig
{
  [JsonPropertyName("version")]
  public int Version { get; init; } = 1;

  [JsonPropertyName("server")]
  public ServerConfig Server { get; init; } = new();

  [JsonPropertyName("ui")]
  public UiPreferences Ui { get; init; } = new();

  [JsonPropertyName("audio")]
  public AudioConfig Audio { get; init; } = new();
}

public record ServerConfig
{
  [JsonPropertyName("last_opencode_url")]
  public string? LastOpencodeUrl { get; init; }

  [JsonPropertyName("auto_start")]
  public bool AutoStart { get; init; } = true;

  [JsonPropertyName("directory_override")]
  public string? DirectoryOverride { get; init; }
}

public record UiPreferences
{
  [JsonPropertyName("font_size")]
  public string FontSize { get; init; } = "Standard";

  [JsonPropertyName("base_font_points")]
  public float BaseFontPoints { get; init; } = 14.0f;

  [JsonPropertyName("chat_density")]
  public string ChatDensity { get; init; } = "Normal";
}

public record AudioConfig
{
  [JsonPropertyName("push_to_talk_key")]
  public string PushToTalkKey { get; init; } = "AltRight";

  [JsonPropertyName("whisper_model_path")]
  public string? WhisperModelPath { get; init; }
}

// ============================================
// MODELS CONFIG (mirrors backend/client-core/src/config/models.rs)
// ============================================

public record ModelsConfig
{
  [JsonPropertyName("providers")]
  public List<ProviderConfig> Providers { get; init; } = new();

  [JsonPropertyName("models")]
  public ModelsSection Models { get; init; } = new();
}

public record ModelsSection
{
  [JsonPropertyName("default_model")]
  public string DefaultModel { get; init; } = "openai/gpt-4";

  [JsonPropertyName("curated")]
  public List<CuratedModel> Curated { get; init; } = new();
}

public record CuratedModel
{
  [JsonPropertyName("name")]
  public string Name { get; init; } = "";

  [JsonPropertyName("provider")]
  public string Provider { get; init; } = "";

  [JsonPropertyName("model_id")]
  public string ModelId { get; init; } = "";

  /// <summary>
  /// Formatted ID for display and API calls (provider/model_id).
  /// </summary>
  public string FullId => $"{Provider}/{ModelId}";
}

public record ProviderConfig
{
  [JsonPropertyName("name")] public string Name { get; init; } = "";

  [JsonPropertyName("display_name")] public string DisplayName { get; init; } = "";

  [JsonPropertyName("api_key_env")] public string ApiKeyEnv { get; init; } = "";

  // Other fields omitted - not needed for Session 11
}