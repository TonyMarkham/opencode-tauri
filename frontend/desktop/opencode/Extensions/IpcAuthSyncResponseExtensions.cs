  namespace OpenCode.Extensions;

  using Opencode;
  using OpenCode.Services;

  public static class IpcAuthSyncResponseExtensions
  {
      public static bool HasSyncedAny(this IpcAuthSyncResponse response)
          => response.Synced.Count > 0;

      public static bool HasFailedAny(this IpcAuthSyncResponse response)
          => response.Failed.Count > 0;

      public static bool HasSkippedAny(this IpcAuthSyncResponse response)
          => response.Skipped.Count > 0;

      public static bool NoKeysFound(this IpcAuthSyncResponse response)
          => response.Synced.Count == 0
             && response.Failed.Count == 0
             && response.Skipped.Count == 0
             && response.ValidationFailed.Count == 0;

      /// <summary>
      /// Convert repeated fields to dictionaries for easier iteration in Razor.
      /// </summary>
      public static Dictionary<string, IpcProviderSyncResult> SyncedProviders(this IpcAuthSyncResponse response)
          => response.Synced.ToDictionary(r => r.Provider);

      public static Dictionary<string, IpcProviderSyncResult> FailedProviders(this IpcAuthSyncResponse response)
          => response.Failed.ToDictionary(r => r.Provider);

      public static Dictionary<string, IpcProviderSyncResult> SkippedProviders(this IpcAuthSyncResponse response)
          => response.Skipped.ToDictionary(r => r.Provider);

      public static Dictionary<string, IpcProviderSyncResult> ValidationFailedProviders(this IpcAuthSyncResponse response)
          => response.ValidationFailed.ToDictionary(r => r.Provider);

      public static string GetSummary(this IpcAuthSyncResponse response, IConfigService configService)
      {
          if (response.NoKeysFound())
              return "No API keys found in .env";

          var parts = new List<string>();
          if (response.Synced.Count > 0)
              parts.Add($"{response.Synced.Count} synced");
          if (response.Failed.Count > 0)
              parts.Add($"{response.Failed.Count} failed");
          if (response.Skipped.Count > 0)
              parts.Add($"{response.Skipped.Count} OAuth");
          if (response.ValidationFailed.Count > 0)
              parts.Add($"{response.ValidationFailed.Count} invalid");

          return string.Join(", ", parts);
      }
  }