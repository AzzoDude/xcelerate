# Client for the human plugin (generated).
# frozen_string_literal: true

require "json"

class Human
  PLUGIN = "human"
  def initialize(browser)
    @browser = browser
  end

  def info()
    args = {}
    raw = @browser.plugin(PLUGIN).invoke("info", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

  def move(x, y)
    args = {"x" => x, "y" => y}
    raw = @browser.plugin(PLUGIN).invoke("move", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

  def click(x, y)
    args = {"x" => x, "y" => y}
    raw = @browser.plugin(PLUGIN).invoke("click", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

  def type(text)
    args = {"text" => text}
    raw = @browser.plugin(PLUGIN).invoke("type", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

  def scroll(deltaY)
    args = {"deltaY" => deltaY}
    raw = @browser.plugin(PLUGIN).invoke("scroll", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

  def delay(minMs = nil, maxMs = nil)
    args = {"minMs" => minMs, "maxMs" => maxMs}
    raw = @browser.plugin(PLUGIN).invoke("delay", JSON.generate(args))
    raw.nil? || raw == "null" ? nil : JSON.parse(raw)
  end

end