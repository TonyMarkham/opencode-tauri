  namespace Opencode.Tests.Services;

  using System.Text.Json;
  using OpenCode.Services;
  using Xunit;

  public class ConfigModelsTests
  {
      private static readonly JsonSerializerOptions s_jsonOptions = new()
      {
          PropertyNameCaseInsensitive = true
      };

      [Fact]
      public void CuratedModel_FullId_FormatsCorrectly()
      {
          // Arrange
          var model = new CuratedModel
          {
              Name = "GPT-4",
              Provider = "openai",
              ModelId = "gpt-4"
          };

          // Act
          var fullId = model.FullId;

          // Assert
          Assert.Equal("openai/gpt-4", fullId);
      }

      [Fact]
      public void ModelsConfig_DeserializesValidJson()
      {
          // Arrange
          var json = """
                     {
                         "providers": [
                             {
                                 "name": "openai",
                                 "display_name": "OpenAI",
                                 "api_key_env": "OPENAI_API_KEY"
                             }
                         ],
                         "models": {
                             "default_model": "openai/gpt-4",
                             "curated": [
                                 {
                                     "name": "GPT-4",
                                     "provider": "openai",
                                     "model_id": "gpt-4"
                                 }
                             ]
                         }
                     }
                     """;

          // Act
          var config = JsonSerializer.Deserialize<ModelsConfig>(json, s_jsonOptions);

          // Assert
          Assert.NotNull(config);
          Assert.Single(config.Providers);
          Assert.Equal("openai", config.Providers[0].Name);
          Assert.Equal("openai/gpt-4", config.Models.DefaultModel);
          Assert.Single(config.Models.Curated);
          Assert.Equal("GPT-4", config.Models.Curated[0].Name);
      }

      [Fact]
      public void ModelsConfig_MissingFields_UsesDefaults()
      {
          // Arrange
          var json = "{}";

          // Act
          var config = JsonSerializer.Deserialize<ModelsConfig>(json, s_jsonOptions);

          // Assert
          Assert.NotNull(config);
          Assert.Empty(config.Providers);
          Assert.Equal("openai/gpt-4", config.Models.DefaultModel); // Default value
          Assert.Empty(config.Models.Curated);
      }

      [Fact]
      public void AppConfig_DeserializesValidJson()
      {
          // Arrange
          var json = """
                     {
                         "version": 1,
                         "server": {
                             "last_opencode_url": "http://localhost:3000",
                             "auto_start": true
                         },
                         "ui": {
                             "font_size": "Large",
                             "base_font_points": 16.0,
                             "chat_density": "Compact"
                         }
                     }
                     """;

          // Act
          var config = JsonSerializer.Deserialize<AppConfig>(json, s_jsonOptions);

          // Assert
          Assert.NotNull(config);
          Assert.Equal(1, config.Version);
          Assert.Equal("http://localhost:3000", config.Server.LastOpencodeUrl);
          Assert.True(config.Server.AutoStart);
          Assert.Equal("Large", config.Ui.FontSize);
          Assert.Equal(16.0f, config.Ui.BaseFontPoints);
      }

      [Fact]
      public void InvalidJson_ThrowsJsonException()
      {
          // Arrange
          var invalidJson = "{ invalid }";

          // Act & Assert
          Assert.Throws<JsonException>(() =>
              JsonSerializer.Deserialize<ModelsConfig>(invalidJson, s_jsonOptions));
      }
  }