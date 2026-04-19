import { UniffiObjectBase } from "./runtime/objects.js";



export interface ComponentMetadata {
  namespace: string;
  packageName: string;
  cdylibName: string;
  nodeEngine: string;
  bundledPrebuilds: boolean;
  manualLoad: boolean;
}

export declare const componentMetadata: Readonly<ComponentMetadata>;

export { ffiMetadata } from "./xcelerate_core-ffi.js";


/**
 * Configuration for the Browser instance.
 */
export interface BrowserConfig {
  /**
   * Whether to run the browser in headless mode.
   */
  "headless": boolean;
  /**
   * Whether to apply stealth patches to the binary.
   */
  "stealth": boolean;
  /**
   * Whether to run the browser as a detached process.
   */
  "detached": boolean;
  /**
   * Optional path to the browser executable.
   */
  "executable_path": string | undefined;
}

export declare class XcelerateError extends globalThis.Error {
  readonly tag: string;
  protected constructor(tag: string, message?: string);
}

export declare class XcelerateErrorWsError extends XcelerateError {
  readonly tag: "WsError";
  constructor(message?: string);
}

export declare class XcelerateErrorSerdeError extends XcelerateError {
  readonly tag: "SerdeError";
  constructor(message?: string);
}

export declare class XcelerateErrorCdpResponseError extends XcelerateError {
  readonly tag: "CdpResponseError";
  constructor(message?: string);
}

export declare class XcelerateErrorHttpError extends XcelerateError {
  readonly tag: "HttpError";
  constructor(message?: string);
}

export declare class XcelerateErrorNotFound extends XcelerateError {
  readonly tag: "NotFound";
  constructor(message?: string);
}

export declare class XcelerateErrorInternalError extends XcelerateError {
  readonly tag: "InternalError";
  constructor(message?: string);
}

/**
 * Represents a browser instance (e.g., Chrome or Edge).
 */
export declare class Browser extends UniffiObjectBase {
  protected constructor();
  static launch(config: BrowserConfig): Promise<Browser>;
  /**
   * Closes the browser and kills the process.
   */
  close(): Promise<void>;
  new_page(url: string): Promise<Page>;
  /**
   * Returns the browser version information.
   */
  version(): Promise<string>;
}

/**
 * Represents an HTML element in the DOM.
 */
export declare class Element extends UniffiObjectBase {
  protected constructor();
  /**
   * Returns the value of a specific attribute.
   */
  attribute(name: string): Promise<string | undefined>;
  /**
   * Clicks the element.
   */
  click(): Promise<Element>;
  /**
   * Focuses the element.
   */
  focus(): Promise<Element>;
  /**
   * Hovers over the element.
   */
  hover(): Promise<Element>;
  /**
   * Returns the inner HTML of the element.
   */
  inner_html(): Promise<string>;
  /**
   * Returns the visible text of the element.
   */
  text(): Promise<string>;
  type_text(text: string): Promise<Element>;
}

export declare class Page extends UniffiObjectBase {
  protected constructor();
  /**
   * Evaluates a script on every new document.
   */
  add_script_to_evaluate_on_new_document(source: string): Promise<string>;
  /**
   * Returns the full HTML content of the page.
   */
  content(): Promise<string>;
  /**
   * Finds an element matching the CSS selector.
   */
  find_element(selector: string): Promise<Element>;
  /**
   * Navigates back in history.
   */
  go_back(): Promise<void>;
  /**
   * Navigates to a URL.
   */
  navigate(url: string): Promise<void>;
  /**
   * Captures a PDF of the page.
   */
  pdf(): Promise<Uint8Array>;
  /**
   * Reloads the page.
   */
  reload(): Promise<void>;
  /**
   * Captures a screenshot of the page as a PNG.
   */
  screenshot(): Promise<Uint8Array>;
  /**
   * Captures a full-page screenshot by overriding device metrics.
   */
  screenshot_full(): Promise<Uint8Array>;
  /**
   * Returns the page title.
   */
  title(): Promise<string>;
  /**
   * Waits for the page to finish loading.
   */
  wait_for_navigation(): Promise<void>;
  /**
   * Waits for an element matching the selector to appear in the DOM.
   */
  wait_for_selector(selector: string): Promise<Element>;
}