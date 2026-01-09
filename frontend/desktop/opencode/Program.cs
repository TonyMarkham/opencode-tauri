using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Opencode;
using OpenCode.Services;
using Radzen;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

builder.Services.AddScoped(sp => new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) });

// Radzen services
builder.Services.AddRadzenComponents();

// Configure IPC client options
builder.Services.Configure<IpcClientOptions>(options =>
{
    options.DefaultRequestTimeout = TimeSpan.FromSeconds(30);
    options.ConnectionTimeout = TimeSpan.FromSeconds(10);
    options.AuthenticationTimeout = TimeSpan.FromSeconds(5);
    options.ShutdownTimeout = TimeSpan.FromSeconds(5);
    options.MaxReceiveBufferSize = 64 * 1024; // 64KB
});

// Register IPC services
builder.Services.AddSingleton<IIpcConfigService, TauriIpcConfigService>();
builder.Services.AddSingleton<IIpcClientMetrics, IpcClientMetrics>();
builder.Services.AddSingleton<IIpcClient, IpcClient>();

// Configure retry policy
builder.Services.Configure<RetryPolicyOptions>(options =>
{
    options.MaxRetries = 3;
    options.InitialDelay = TimeSpan.FromMilliseconds(100);
    options.MaxDelay = TimeSpan.FromSeconds(2);
    options.BackoffMultiplier = 2.0;
    options.AddJitter = true;
});

// Register config services
builder.Services.AddSingleton<IRetryPolicy, RetryPolicy>();
builder.Services.AddSingleton<IConfigService, ConfigService>();

// Register auth sync services
builder.Services.AddSingleton<IAuthSyncMetrics, AuthSyncMetrics>();
builder.Services.AddSingleton<IAuthSyncService, AuthSyncService>();

await builder.Build().RunAsync();
