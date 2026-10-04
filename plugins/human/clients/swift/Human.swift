// Client for the human plugin (generated).
import Foundation

public let pluginName = "human"

public struct Info: Codable, Sendable {
    public var name: String?
    public var enabled: Bool?
    public var ops: [String]?
    enum CodingKeys: String, CodingKey {
        case name = "name"
        case enabled = "enabled"
        case ops = "ops"
    }
}

public struct Move: Codable, Sendable {
    public var moved: Bool?
    public var x: Double?
    public var y: Double?
    enum CodingKeys: String, CodingKey {
        case moved = "moved"
        case x = "x"
        case y = "y"
    }
}

public struct Click: Codable, Sendable {
    public var clicked: Bool?
    public var x: Double?
    public var y: Double?
    enum CodingKeys: String, CodingKey {
        case clicked = "clicked"
        case x = "x"
        case y = "y"
    }
}

public struct Type: Codable, Sendable {
    public var typed: Int?
    enum CodingKeys: String, CodingKey {
        case typed = "typed"
    }
}

public struct Scroll: Codable, Sendable {
    public var scrolled: Double?
    enum CodingKeys: String, CodingKey {
        case scrolled = "scrolled"
    }
}

public struct Delay: Codable, Sendable {
    public var sleptMs: Int?
    enum CodingKeys: String, CodingKey {
        case sleptMs = "sleptMs"
    }
}

struct InfoRequest: Encodable {
}

struct MoveRequest: Encodable {
    let x: Double
    let y: Double
}

struct ClickRequest: Encodable {
    let x: Double
    let y: Double
}

struct TypeRequest: Encodable {
    let text: String
}

struct ScrollRequest: Encodable {
    let deltaY: Double
}

struct DelayRequest: Encodable {
    let minMs: Int?
    let maxMs: Int?
}

public struct Human {
    let browser: Browser
    public init(browser: Browser) { self.browser = browser }

    public func info() async throws -> Info {
        let json = String(data: try JSONEncoder().encode(InfoRequest()), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "info", argsJson: json)
        return try JSONDecoder().decode(Info.self, from: Data(raw.utf8))
    }

    public func move(x: Double, y: Double) async throws -> Move {
        let json = String(data: try JSONEncoder().encode(MoveRequest(x: x, y: y)), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "move", argsJson: json)
        return try JSONDecoder().decode(Move.self, from: Data(raw.utf8))
    }

    public func click(x: Double, y: Double) async throws -> Click {
        let json = String(data: try JSONEncoder().encode(ClickRequest(x: x, y: y)), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "click", argsJson: json)
        return try JSONDecoder().decode(Click.self, from: Data(raw.utf8))
    }

    public func type(text: String) async throws -> Type {
        let json = String(data: try JSONEncoder().encode(TypeRequest(text: text)), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "type", argsJson: json)
        return try JSONDecoder().decode(Type.self, from: Data(raw.utf8))
    }

    public func scroll(deltaY: Double) async throws -> Scroll {
        let json = String(data: try JSONEncoder().encode(ScrollRequest(deltaY: deltaY)), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "scroll", argsJson: json)
        return try JSONDecoder().decode(Scroll.self, from: Data(raw.utf8))
    }

    public func delay(minMs: Int? = nil, maxMs: Int? = nil) async throws -> Delay {
        let json = String(data: try JSONEncoder().encode(DelayRequest(minMs: minMs, maxMs: maxMs)), encoding: .utf8)!
        let raw = try await browser.plugin(name: pluginName).invoke(op: "delay", argsJson: json)
        return try JSONDecoder().decode(Delay.self, from: Data(raw.utf8))
    }

}