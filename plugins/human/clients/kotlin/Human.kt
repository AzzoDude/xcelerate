// Client for the human plugin (generated).
package xcelerate.plugins

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import uniffi.xcelerate.Browser

data class Info(
    @SerializedName("name") val name: String? = null,
    @SerializedName("enabled") val enabled: Boolean? = null,
    @SerializedName("ops") val ops: List<String>? = null,
)

data class Move(
    @SerializedName("moved") val moved: Boolean? = null,
    @SerializedName("x") val x: Double? = null,
    @SerializedName("y") val y: Double? = null,
)

data class Click(
    @SerializedName("clicked") val clicked: Boolean? = null,
    @SerializedName("x") val x: Double? = null,
    @SerializedName("y") val y: Double? = null,
)

data class Type(
    @SerializedName("typed") val typed: Long? = null,
)

data class Scroll(
    @SerializedName("scrolled") val scrolled: Double? = null,
)

data class Delay(
    @SerializedName("sleptMs") val sleptMs: Long? = null,
)

class Human(private val browser: Browser) {
    companion object {
        const val PLUGIN = "human"
        private val GSON = Gson()
    }

    suspend fun info(): Info {
        val json = "{}"
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("info", json), Info::class.java)
    }

    suspend fun move(x: Double?, y: Double?): Move {
        val json = GSON.toJson(mapOf("x" to x, "y" to y))
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("move", json), Move::class.java)
    }

    suspend fun click(x: Double?, y: Double?): Click {
        val json = GSON.toJson(mapOf("x" to x, "y" to y))
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("click", json), Click::class.java)
    }

    suspend fun type(text: String?): Type {
        val json = GSON.toJson(mapOf("text" to text))
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("type", json), Type::class.java)
    }

    suspend fun scroll(deltaY: Double?): Scroll {
        val json = GSON.toJson(mapOf("deltaY" to deltaY))
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("scroll", json), Scroll::class.java)
    }

    suspend fun delay(minMs: Long?, maxMs: Long?): Delay {
        val json = GSON.toJson(mapOf("minMs" to minMs, "maxMs" to maxMs))
        return GSON.fromJson(browser.plugin(PLUGIN).invoke("delay", json), Delay::class.java)
    }

}