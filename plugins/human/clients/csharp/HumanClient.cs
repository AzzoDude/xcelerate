// Client for the human plugin (generated).
using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Human;

public sealed record Info
{
    [JsonPropertyName("name")]
    public string Name { get; init; }
    [JsonPropertyName("enabled")]
    public bool Enabled { get; init; }
    [JsonPropertyName("ops")]
    public List<string> Ops { get; init; }
}

public sealed record Move
{
    [JsonPropertyName("moved")]
    public bool Moved { get; init; }
    [JsonPropertyName("x")]
    public double X { get; init; }
    [JsonPropertyName("y")]
    public double Y { get; init; }
}

public sealed record Click
{
    [JsonPropertyName("clicked")]
    public bool Clicked { get; init; }
    [JsonPropertyName("x")]
    public double X { get; init; }
    [JsonPropertyName("y")]
    public double Y { get; init; }
}

public sealed record Type
{
    [JsonPropertyName("typed")]
    public long Typed { get; init; }
}

public sealed record Scroll
{
    [JsonPropertyName("scrolled")]
    public double Scrolled { get; init; }
}

public sealed record Delay
{
    [JsonPropertyName("sleptMs")]
    public long SleptMs { get; init; }
}

public sealed class HumanClient
{
    public const string Plugin = "human";
    static readonly JsonSerializerOptions Options = new()
    {
        // The wire keys are exactly the field names; no renaming policy.
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };
    readonly Browser _browser;
    public HumanClient(Browser browser) => _browser = browser;

    public async Task<Info> InfoAsync()
    {
        var raw = await _browser.Plugin(Plugin).Invoke("info", JsonSerializer.Serialize(new { }, Options));
        return JsonSerializer.Deserialize<Info>(raw, Options)!;
    }

    public async Task<Move> MoveAsync(double x, double y)
    {
        var raw = await _browser.Plugin(Plugin).Invoke("move", JsonSerializer.Serialize(new { x, y }, Options));
        return JsonSerializer.Deserialize<Move>(raw, Options)!;
    }

    public async Task<Click> ClickAsync(double x, double y)
    {
        var raw = await _browser.Plugin(Plugin).Invoke("click", JsonSerializer.Serialize(new { x, y }, Options));
        return JsonSerializer.Deserialize<Click>(raw, Options)!;
    }

    public async Task<Type> TypeAsync(string text)
    {
        var raw = await _browser.Plugin(Plugin).Invoke("type", JsonSerializer.Serialize(new { text }, Options));
        return JsonSerializer.Deserialize<Type>(raw, Options)!;
    }

    public async Task<Scroll> ScrollAsync(double deltaY)
    {
        var raw = await _browser.Plugin(Plugin).Invoke("scroll", JsonSerializer.Serialize(new { deltaY }, Options));
        return JsonSerializer.Deserialize<Scroll>(raw, Options)!;
    }

    public async Task<Delay> DelayAsync(long? minMs = null, long? maxMs = null)
    {
        var raw = await _browser.Plugin(Plugin).Invoke("delay", JsonSerializer.Serialize(new { minMs, maxMs }, Options));
        return JsonSerializer.Deserialize<Delay>(raw, Options)!;
    }

}