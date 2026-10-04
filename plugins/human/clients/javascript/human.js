// Client for the human plugin (generated).
/* eslint-disable */

export const PLUGIN = "human";

export interface Info {
  name?: string;
  enabled?: boolean;
  ops?: string[];
}

export interface Move {
  moved?: boolean;
  x?: number;
  y?: number;
}

export interface Click {
  clicked?: boolean;
  x?: number;
  y?: number;
}

export interface Type {
  typed?: number;
}

export interface Scroll {
  scrolled?: number;
}

export interface Delay {
  sleptMs?: number;
}

export class Human {
  constructor(private readonly browser: any) {}

  async info(): Promise<Info> {
    const args = {  };
    const raw = await this.browser.plugin(PLUGIN).invoke("info", JSON.stringify(args));
    return JSON.parse(raw);
  }

  async move(x: number, y: number): Promise<Move> {
    const args = { x, y };
    const raw = await this.browser.plugin(PLUGIN).invoke("move", JSON.stringify(args));
    return JSON.parse(raw);
  }

  async click(x: number, y: number): Promise<Click> {
    const args = { x, y };
    const raw = await this.browser.plugin(PLUGIN).invoke("click", JSON.stringify(args));
    return JSON.parse(raw);
  }

  async type(text: string): Promise<Type> {
    const args = { text };
    const raw = await this.browser.plugin(PLUGIN).invoke("type", JSON.stringify(args));
    return JSON.parse(raw);
  }

  async scroll(deltaY: number): Promise<Scroll> {
    const args = { deltaY };
    const raw = await this.browser.plugin(PLUGIN).invoke("scroll", JSON.stringify(args));
    return JSON.parse(raw);
  }

  async delay(minMs?: number, maxMs?: number): Promise<Delay> {
    const args = { minMs, maxMs };
    const raw = await this.browser.plugin(PLUGIN).invoke("delay", JSON.stringify(args));
    return JSON.parse(raw);
  }

}