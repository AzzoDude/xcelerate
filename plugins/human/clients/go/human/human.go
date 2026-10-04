// Client for the human plugin (generated).
package human

import "encoding/json"

const Plugin = "human"

// Browser is the part of the engine binding this client needs.
type Browser interface { Plugin(name string) PluginHandle }
type PluginHandle interface { Invoke(op, argsJSON string) (string, error) }

type Info struct {
    Name string `json:"name,omitempty"`
    Enabled bool `json:"enabled,omitempty"`
    Ops []string `json:"ops,omitempty"`
}

type Move struct {
    Moved bool `json:"moved,omitempty"`
    X float64 `json:"x,omitempty"`
    Y float64 `json:"y,omitempty"`
}

type Click struct {
    Clicked bool `json:"clicked,omitempty"`
    X float64 `json:"x,omitempty"`
    Y float64 `json:"y,omitempty"`
}

type Type struct {
    Typed int64 `json:"typed,omitempty"`
}

type Scroll struct {
    Scrolled float64 `json:"scrolled,omitempty"`
}

type Delay struct {
    SleptMs int64 `json:"sleptMs,omitempty"`
}

type Human struct { browser Browser }
func New(browser Browser) *Human { return &Human{browser} }

func (c *Human) Info() (Info, error) {
    args, err := json.Marshal(map[string]any{})
    if err != nil { return *new(Info), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("info", string(args))
    if err != nil { return *new(Info), err }
    var out Info
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}

func (c *Human) Move(x float64, y float64) (Move, error) {
    args, err := json.Marshal(map[string]any{"x": x, "y": y})
    if err != nil { return *new(Move), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("move", string(args))
    if err != nil { return *new(Move), err }
    var out Move
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}

func (c *Human) Click(x float64, y float64) (Click, error) {
    args, err := json.Marshal(map[string]any{"x": x, "y": y})
    if err != nil { return *new(Click), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("click", string(args))
    if err != nil { return *new(Click), err }
    var out Click
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}

func (c *Human) Type(text string) (Type, error) {
    args, err := json.Marshal(map[string]any{"text": text})
    if err != nil { return *new(Type), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("type", string(args))
    if err != nil { return *new(Type), err }
    var out Type
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}

func (c *Human) Scroll(deltaY float64) (Scroll, error) {
    args, err := json.Marshal(map[string]any{"deltaY": deltaY})
    if err != nil { return *new(Scroll), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("scroll", string(args))
    if err != nil { return *new(Scroll), err }
    var out Scroll
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}

func (c *Human) Delay(minMs int64, maxMs int64) (Delay, error) {
    args, err := json.Marshal(map[string]any{"minMs": minMs, "maxMs": maxMs})
    if err != nil { return *new(Delay), err }
    raw, err := c.browser.Plugin(Plugin).Invoke("delay", string(args))
    if err != nil { return *new(Delay), err }
    var out Delay
    err = json.Unmarshal([]byte(raw), &out)
    return out, err
}
