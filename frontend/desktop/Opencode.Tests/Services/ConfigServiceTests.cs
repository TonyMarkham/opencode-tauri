  using System;
  using System.Threading;
  using System.Threading.Tasks;

  namespace Opencode.Tests.Services;

  using Microsoft.Extensions.Logging;
  using Moq;
  using OpenCode.Services;
  using OpenCode.Services.Exceptions;
  using Xunit;

  public class ConfigServiceTests
  {
      private readonly Mock<IIpcClient> _ipcClientMock;
      private readonly Mock<IRetryPolicy> _retryPolicyMock;
      private readonly Mock<ILogger<ConfigService>> _loggerMock;
      private readonly ConfigService _configService;

      public ConfigServiceTests()
      {
          _ipcClientMock = new Mock<IIpcClient>();
          _retryPolicyMock = new Mock<IRetryPolicy>();
          _loggerMock = new Mock<ILogger<ConfigService>>();

          _ipcClientMock.Setup(x => x.IsConnected).Returns(true);

          _configService = new ConfigService(
              _ipcClientMock.Object,
              _retryPolicyMock.Object,
              _loggerMock.Object);
      }

      [Fact]
      public void InitialState_IsNotLoaded()
      {
          Assert.Equal(ConfigLoadState.NotLoaded, _configService.State);
          Assert.Null(_configService.ModelsConfig);
          Assert.Null(_configService.AppConfig);
      }

      [Fact]
      public async Task AfterSuccessfulLoad_StateIsLoaded()
      {
          // Arrange
          var appConfig = new AppConfig();
          var modelsConfig = new ModelsConfig();

          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ReturnsAsync((appConfig, modelsConfig));

          // Act
          await _configService.RefreshAsync();

          // Assert
          Assert.Equal(ConfigLoadState.Loaded, _configService.State);
          Assert.NotNull(_configService.ModelsConfig);
          Assert.NotNull(_configService.AppConfig);
          Assert.NotNull(_configService.LastLoadedAt);
      }

      [Fact]
      public async Task AfterFailedLoad_WithNoCache_StateIsError()
      {
          // Arrange
          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ThrowsAsync(new IpcTimeoutException(1, TimeSpan.FromSeconds(1)));

          // Act
          await _configService.RefreshAsync();

          // Assert
          Assert.Equal(ConfigLoadState.Error, _configService.State);
          Assert.Null(_configService.ModelsConfig);
          Assert.NotNull(_configService.ErrorMessage);
      }

      [Fact]
      public async Task AfterFailedLoad_WithCache_StateIsStale()
      {
          // Arrange - first load succeeds
          var appConfig = new AppConfig();
          var modelsConfig = new ModelsConfig();

          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ReturnsAsync((appConfig, modelsConfig));

          await _configService.RefreshAsync();

          // Act - second load fails
          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ThrowsAsync(new IpcTimeoutException(2, TimeSpan.FromSeconds(1)));

          await _configService.RefreshAsync();

          // Assert - state is stale but data is preserved
          Assert.Equal(ConfigLoadState.Stale, _configService.State);
          Assert.NotNull(_configService.ModelsConfig); // Cached data preserved
          Assert.NotNull(_configService.ErrorMessage);
      }

      [Fact]
      public async Task GetConfig_WithinMaxAge_ReturnsCachedImmediately()
      {
          // Arrange
          var appConfig = new AppConfig();
          var modelsConfig = new ModelsConfig();

          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ReturnsAsync((appConfig, modelsConfig));

          await _configService.RefreshAsync();

          // Act - get config again immediately
          var (app, models) = await _configService.GetConfigAsync(TimeSpan.FromMinutes(1));

          // Assert - should return cached without calling IPC again
          Assert.Same(appConfig, app);
          Assert.Same(modelsConfig, models);

          // Verify IPC was only called once (during RefreshAsync)
          _retryPolicyMock.Verify(
              x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()),
              Times.Once);
      }

      [Fact]
      public async Task OnConfigChanged_FiresEvent()
      {
          // Arrange
          var eventFired = false;
          ConfigChangedEventArgs? eventArgs = null;

          _configService.ConfigChanged += (sender, args) =>
          {
              eventFired = true;
              eventArgs = args;
          };

          var appConfig = new AppConfig();
          var modelsConfig = new ModelsConfig();

          _retryPolicyMock
              .Setup(x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()))
              .ReturnsAsync((appConfig, modelsConfig));

          // Act
          await _configService.RefreshAsync();

          // Assert
          Assert.True(eventFired);
          Assert.NotNull(eventArgs);
          Assert.Equal(ConfigLoadState.Loaded, eventArgs.State);
      }

      [Fact]
      public async Task WhenDisconnected_DoesNotAttemptLoad()
      {
          // Arrange
          _ipcClientMock.Setup(x => x.IsConnected).Returns(false);

          // Act
          await _configService.RefreshAsync();

          // Assert
          Assert.Equal(ConfigLoadState.Error, _configService.State);
          Assert.Contains("not connected", _configService.ErrorMessage?.ToLower() ?? "");

          // Verify retry policy was never called
          _retryPolicyMock.Verify(
              x => x.ExecuteAsync(
                  It.IsAny<Func<CancellationToken, Task<(AppConfig, ModelsConfig)>>>(),
                  It.IsAny<Func<Exception, bool>>(),
                  It.IsAny<CancellationToken>()),
              Times.Never);
      }
  }