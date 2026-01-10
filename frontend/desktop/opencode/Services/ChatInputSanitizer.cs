  using System.Text.RegularExpressions;

  namespace OpenCode.Services;

  /// <summary>
  /// Sanitizes user input to prevent XSS and injection attacks.
  /// Applied before display and before sending to backend.
  /// </summary>
  public static partial class ChatInputSanitizer
  {
      // Regex patterns compiled at build time for performance
      [GeneratedRegex(@"<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>", RegexOptions.IgnoreCase)]
      private static partial Regex ScriptTagRegex();

      [GeneratedRegex(@"javascript:", RegexOptions.IgnoreCase)]
      private static partial Regex JavaScriptProtocolRegex();

      [GeneratedRegex(@"on\w+\s*=", RegexOptions.IgnoreCase)]
      private static partial Regex EventHandlerRegex();

      /// <summary>
      /// Sanitize input for safe display and transmission.
      /// </summary>
      public static string Sanitize(string? input)
      {
          if (string.IsNullOrEmpty(input))
              return string.Empty;

          var result = input;

          // Remove script tags
          result = ScriptTagRegex().Replace(result, string.Empty);

          // Remove javascript: protocol
          result = JavaScriptProtocolRegex().Replace(result, string.Empty);

          // Remove event handlers (onclick=, onerror=, etc.)
          result = EventHandlerRegex().Replace(result, string.Empty);

          // Trim excessive whitespace (but preserve intentional newlines)
          result = NormalizeWhitespace(result);

          return result;
      }

      /// <summary>
      /// Check if input contains potentially dangerous content.
      /// </summary>
      public static bool ContainsDangerousContent(string? input)
      {
          if (string.IsNullOrEmpty(input))
              return false;

          return ScriptTagRegex().IsMatch(input)
              || JavaScriptProtocolRegex().IsMatch(input)
              || EventHandlerRegex().IsMatch(input);
      }

      /// <summary>
      /// Normalize whitespace while preserving intentional formatting.
      /// </summary>
      private static string NormalizeWhitespace(string input)
      {
          // Replace multiple spaces with single space (but keep newlines)
          var lines = input.Split('\n');
          for (int i = 0; i < lines.Length; i++)
          {
              // Collapse multiple spaces to one, trim each line
              lines[i] = Regex.Replace(lines[i], @" {2,}", " ").Trim();
          }

          // Collapse more than 2 consecutive newlines to 2
          var result = string.Join("\n", lines);
          result = Regex.Replace(result, @"\n{3,}", "\n\n");

          return result.Trim();
      }

      /// <summary>
      /// HTML encode for safe display (when rendering user content).
      /// </summary>
      public static string HtmlEncode(string? input)
      {
          if (string.IsNullOrEmpty(input))
              return string.Empty;

          return System.Net.WebUtility.HtmlEncode(input);
      }
  }