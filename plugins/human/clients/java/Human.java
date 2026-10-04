// Client for the human plugin (generated).
package xcelerate.plugins;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import com.google.gson.Gson;
import com.google.gson.annotations.SerializedName;
import uniffi.xcelerate.Browser;

public record Info(@SerializedName("name") String name, @SerializedName("enabled") boolean enabled, @SerializedName("ops") List<String> ops) {}

public record Move(@SerializedName("moved") boolean moved, @SerializedName("x") double x, @SerializedName("y") double y) {}

public record Click(@SerializedName("clicked") boolean clicked, @SerializedName("x") double x, @SerializedName("y") double y) {}

public record Type(@SerializedName("typed") long typed) {}

public record Scroll(@SerializedName("scrolled") double scrolled) {}

public record Delay(@SerializedName("sleptMs") long sleptMs) {}

public final class Human {
    public static final String PLUGIN = "human";
    private static final Gson GSON = new Gson();
    private final Browser browser;
    public Human(Browser browser) { this.browser = browser; }

    public CompletableFuture<Info> info() {
        Map<String, Object> args = new LinkedHashMap<>();
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("info", json)
            .thenApply(raw -> GSON.fromJson(raw, Info.class));
    }

    public CompletableFuture<Move> move(double x, double y) {
        Map<String, Object> args = new LinkedHashMap<>();
        args.put("x", x);
        args.put("y", y);
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("move", json)
            .thenApply(raw -> GSON.fromJson(raw, Move.class));
    }

    public CompletableFuture<Click> click(double x, double y) {
        Map<String, Object> args = new LinkedHashMap<>();
        args.put("x", x);
        args.put("y", y);
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("click", json)
            .thenApply(raw -> GSON.fromJson(raw, Click.class));
    }

    public CompletableFuture<Type> type(String text) {
        Map<String, Object> args = new LinkedHashMap<>();
        args.put("text", text);
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("type", json)
            .thenApply(raw -> GSON.fromJson(raw, Type.class));
    }

    public CompletableFuture<Scroll> scroll(double deltaY) {
        Map<String, Object> args = new LinkedHashMap<>();
        args.put("deltaY", deltaY);
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("scroll", json)
            .thenApply(raw -> GSON.fromJson(raw, Scroll.class));
    }

    public CompletableFuture<Delay> delay(long minMs, long maxMs) {
        Map<String, Object> args = new LinkedHashMap<>();
        args.put("minMs", minMs);
        args.put("maxMs", maxMs);
        String json = GSON.toJson(args);
        return browser.plugin(PLUGIN).invoke("delay", json)
            .thenApply(raw -> GSON.fromJson(raw, Delay.class));
    }

}