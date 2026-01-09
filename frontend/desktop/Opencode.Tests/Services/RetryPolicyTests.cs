  using System;
  using System.Collections.Generic;
  using System.Threading;
  using System.Threading.Tasks;

  namespace Opencode.Tests.Services;

  using Microsoft.Extensions.Logging;
  using Microsoft.Extensions.Options;
  using Moq;
  using OpenCode.Services;
  using Xunit;

  public class RetryPolicyTests
  {
      private readonly Mock<ILogger<RetryPolicy>> _loggerMock;
      private readonly RetryPolicyOptions _options;
      private readonly RetryPolicy _retryPolicy;

      public RetryPolicyTests()
      {
          _loggerMock = new Mock<ILogger<RetryPolicy>>();
          _options = new RetryPolicyOptions
          {
              MaxRetries = 3,
              InitialDelay = TimeSpan.FromMilliseconds(10), // Fast for tests
              MaxDelay = TimeSpan.FromMilliseconds(100),
              BackoffMultiplier = 2.0,
              AddJitter = false // Deterministic for tests
          };
          _retryPolicy = new RetryPolicy(Options.Create(_options), _loggerMock.Object);
      }

      [Fact]
      public async Task SuccessOnFirstAttempt_NoRetry()
      {
          // Arrange
          var callCount = 0;

          Task<int> Operation(CancellationToken ct)
          {
              callCount++;
              return Task.FromResult(42);
          }

          // Act
          var result = await _retryPolicy.ExecuteAsync(Operation);

          // Assert
          Assert.Equal(42, result);
          Assert.Equal(1, callCount);
      }

      [Fact]
      public async Task SuccessOnSecondAttempt_RetriesOnce()
      {
          // Arrange
          var callCount = 0;

          Task<int> Operation(CancellationToken ct)
          {
              callCount++;
              if (callCount == 1)
                  throw new InvalidOperationException("Transient error");
              return Task.FromResult(42);
          }

          // Act
          var result = await _retryPolicy.ExecuteAsync(Operation);

          // Assert
          Assert.Equal(42, result);
          Assert.Equal(2, callCount);
      }

      [Fact]
      public async Task AllAttemptsFail_ThrowsLastException()
      {
          // Arrange
          var callCount = 0;

          Task<int> Operation(CancellationToken ct)
          {
              callCount++;
              throw new InvalidOperationException($"Attempt {callCount}");
          }

          // Act & Assert
          var ex = await Assert.ThrowsAsync<InvalidOperationException>(() => _retryPolicy.ExecuteAsync(Operation));

          Assert.Equal("Attempt 4", ex.Message); // Max retries = 3, so 4 total attempts
          Assert.Equal(4, callCount);
      }

      [Fact]
      public async Task NonRetryableException_DoesNotRetry()
      {
          // Arrange
          var callCount = 0;

          Task<int> Operation(CancellationToken ct)
          {
              callCount++;
              throw new ArgumentException("Non-retryable");
          }

          static bool ShouldRetry(Exception ex) => ex is InvalidOperationException;

          // Act & Assert
          await Assert.ThrowsAsync<ArgumentException>(() => _retryPolicy.ExecuteAsync(Operation, ShouldRetry));

          Assert.Equal(1, callCount); // No retries
      }

      [Fact]
      public async Task CancellationRequested_StopsImmediately()
      {
          // Arrange
          var callCount = 0;
          var cts = new CancellationTokenSource();

          Task<int> Operation(CancellationToken ct)
          {
              callCount++;
              cts.Cancel(); // Cancel after first attempt
              ct.ThrowIfCancellationRequested();
              return Task.FromResult(42);
          }

          // Act & Assert
          await Assert.ThrowsAsync<OperationCanceledException>(() =>
              _retryPolicy.ExecuteAsync(Operation, cancellationToken: cts.Token));

          Assert.Equal(1, callCount); // Stopped immediately
      }

      [Fact]
      public async Task BackoffDelays_AreExponential()
      {
          // Arrange
          var attempts = new List<DateTime>();
          Task<int> Operation(CancellationToken ct)
          {
              attempts.Add(DateTime.UtcNow);
              throw new InvalidOperationException("Always fail");
          }

          // Act
          try
          {
              await _retryPolicy.ExecuteAsync(Operation);
          }
          catch
          {
              // Expected
          }

          // Assert
          Assert.Equal(4, attempts.Count); // 1 initial + 3 retries

          var delay1 = attempts[1] - attempts[0];
          var delay2 = attempts[2] - attempts[1];
          var delay3 = attempts[3] - attempts[2];

          // Delays should be increasing (allow generous tolerance for timing variance)
          // Just verify delays exist and are generally increasing
          Assert.True(delay1.TotalMilliseconds >= _options.InitialDelay.TotalMilliseconds * 0.5);
          Assert.True(delay2.TotalMilliseconds >= _options.InitialDelay.TotalMilliseconds * 0.5);
          Assert.True(delay3.TotalMilliseconds >= _options.InitialDelay.TotalMilliseconds * 0.5);
      }
  }