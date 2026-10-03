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

export { ffiMetadata } from "./xcelerate-ffi.js";


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

export declare class XcelerateErrorUnsupported extends XcelerateError {
  readonly tag: "Unsupported";
  constructor(message?: string);
}

/**
 * Represents a browser instance (e.g., Chrome or Edge).
 */
export declare class Browser extends UniffiObjectBase {
  protected constructor();
  static launch(config: BrowserConfig): Promise<Browser>;
  /**
   * Returns the browser context ids as a JSON array.
   */
  browser_contexts(): Promise<string>;
  /**
   * Returns the browser version info as JSON.
   */
  capabilities(): Promise<string>;
  /**
   * Closes the browser and kills the process.
   */
  close(): Promise<void>;
  /**
   * Returns all browser cookies as a JSON array.
   */
  cookies(): Promise<string>;
  /**
   * Deletes cookies with the given name.
   */
  delete_cookie(name: string): Promise<void>;
  /**
   * Returns the registered event names.
   */
  event_names(): Promise<Array<string>>;
  /**
   * Grants permissions (JSON array) to an origin.
   */
  grant_permissions(origin: string, permissions_json: string): Promise<void>;
  /**
   * Whether the underlying connection is alive.
   */
  is_connected(): Promise<boolean>;
  /**
   * Whether an event name is registered.
   */
  listens_to(event_name: string): Promise<boolean>;
  /**
   * Creates a new (incognito) browser context and returns its id.
   */
  new_context(): Promise<string>;
  new_page(url: string): Promise<Page>;
  /**
   * Registers interest in a root-session CDP event.
   */
  on(event_name: string): Promise<void>;
  /**
   * Alias for [`Browser::on`].
   */
  once(event_name: string): Promise<void>;
  /**
   * Removes every registered listener.
   */
  remove_all_listeners(): Promise<void>;
  /**
   * Removes a single registered listener.
   */
  remove_listener(event_name: string): Promise<void>;
  /**
   * Resets all permission overrides.
   */
  reset_permissions(): Promise<void>;
  /**
   * Sets a cookie from a JSON object.
   */
  set_cookie(cookie_json: string): Promise<void>;
  /**
   * Sets the download directory for the browser.
   */
  set_download_behavior(path: string): Promise<void>;
  /**
   * Starts CDP tracing.
   */
  start_tracing(): Promise<void>;
  /**
   * Stops CDP tracing.
   */
  stop_tracing(): Promise<void>;
  /**
   * Returns the current targets as a JSON array (`Target.getTargets`).
   */
  targets(): Promise<string>;
  /**
   * Returns the browser's user agent.
   */
  user_agent(): Promise<string>;
  /**
   * Returns the browser version information.
   */
  version(): Promise<string>;
  /**
   * Waits for the next root-session CDP event named `event_name`.
   */
  wait_for_event(event_name: string, timeout_ms: bigint | number): Promise<string>;
  /**
   * [`Browser::wait_for_event`] with the default 30s timeout.
   */
  wait_for_event_default(event_name: string): Promise<string>;
  /**
   * The WebSocket endpoint Chrome was launched with.
   */
  ws_endpoint(): string;
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
   * Like [`Element::call_json`] but coerces the result to a bool.
   */
  call_bool(function_: string, args_json: string): Promise<boolean>;
  /**
   * Calls a JS function on this element with JSON-encoded arguments.
   */
  call_json(function_: string, args_json: string): Promise<string>;
  /**
   * Runs a JS function against the first descendant matching `selector`.
   */
  call_on_selector(selector: string, expression: string): Promise<string>;
  /**
   * Runs a JS function against every descendant matching `selector`.
   */
  call_on_selector_all(selector: string, expression: string): Promise<string>;
  /**
   * Like [`Element::call_json`] but coerces the result to a string.
   */
  call_string(function_: string, args_json: string): Promise<string>;
  /**
   * Clicks the element.
   */
  click(): Promise<Element>;
  /**
   * Clicks the element using realistic mouse movement and CDP input events.
   */
  click_stealth(): Promise<Element>;
  /**
   * Number of elements this handle represents (always 1).
   */
  count(): Promise<bigint | number>;
  /**
   * Releases the underlying remote object handle.
   */
  dispose(): Promise<void>;
  /**
   * Calls a function on this element and coerces the result to a bool.
   */
  evaluate_bool(function_: string): Promise<boolean>;
  /**
   * Calls a JS function on this element and returns the resulting node.
   */
  evaluate_handle(function_: string): Promise<Element>;
  /**
   * Calls a function on this element; the result is returned as JSON text.
   */
  evaluate_json(function_: string): Promise<string>;
  /**
   * Calls a function on this element and coerces the result to a string.
   */
  evaluate_string(function_: string): Promise<string>;
  /**
   * Focuses the element.
   */
  focus(): Promise<Element>;
  /**
   * Finds a descendant form control by its `<label>` text.
   */
  get_by_label(label: string): Promise<Element>;
  /**
   * Finds a descendant by ARIA role.
   */
  get_by_role(role: string): Promise<Element>;
  /**
   * Finds a descendant whose text contains `text`.
   */
  get_by_text(text: string): Promise<Element>;
  /**
   * Returns this element's enumerable properties as a JSON object.
   */
  get_properties(): Promise<string>;
  /**
   * Hovers over the element.
   */
  hover(): Promise<Element>;
  /**
   * Hovers over the element using realistic mouse movement.
   */
  hover_stealth(): Promise<Element>;
  /**
   * Returns the inner HTML of the element.
   */
  inner_html(): Promise<string>;
  /**
   * Focuses the element and presses a key.
   */
  press(key: string): Promise<void>;
  /**
   * Returns the first descendant matching `selector` as an [`Element`].
   */
  query_selector(selector: string): Promise<Element>;
  /**
   * Returns every descendant matching `selector`.
   *
   * Resolves the whole node list with a single `Runtime.getProperties` call
   * rather than one `evaluate` per match.
   */
  query_selector_all(selector: string): Promise<Array<Element>>;
  /**
   * Finds a descendant by attribute value.
   */
  query_selector_attr(attribute: string, value: string): Promise<Element>;
  /**
   * Finds a descendant matching an XPath expression.
   */
  query_selector_xpath(xpath: string): Promise<Element>;
  /**
   * Captures a PNG screenshot cropped to this element.
   */
  screenshot(): Promise<Uint8Array>;
  /**
   * Captures a base64 PNG screenshot cropped to this element.
   */
  screenshot_base64(): Promise<string>;
  /**
   * Selects options by value or label on this `<select>` element.
   */
  select_option(values_json: string): Promise<void>;
  /**
   * Sets the files of this `<input type="file">` element.
   */
  set_input_files(files_json: string): Promise<void>;
  /**
   * Returns the visible text of the element.
   */
  text(): Promise<string>;
  type_text(text: string): Promise<Element>;
  /**
   * Waits for a descendant matching `selector` to appear.
   */
  wait_for_selector(selector: string): Promise<Element>;
}

export declare class Page extends UniffiObjectBase {
  protected constructor();
  /**
   * Activates this page's target.
   */
  activate(): Promise<void>;
  /**
   * Activates the given target (window/tab).
   */
  activate_target(target_id: string): Promise<void>;
  /**
   * Evaluates a script on every new document.
   */
  add_script_to_evaluate_on_new_document(source: string): Promise<string>;
  /**
   * Injects a `<style>` element and returns the injected content.
   */
  add_style_tag(content: string): Promise<string>;
  /**
   * Sets the credentials used to answer HTTP auth challenges.
   */
  authenticate(username: string, password: string): Promise<void>;
  /**
   * Brings the page to the front.
   */
  bring_to_front(): Promise<void>;
  /**
   * Like [`Page::call_json`] but coerces the result to a bool.
   */
  call_bool(function_: string, args_json: string): Promise<boolean>;
  /**
   * Calls a JS function with JSON-encoded arguments, returning JSON text.
   *
   * This is the shim the adapters use to express the broad upstream surface
   * as data (a function body per method) rather than a core method per member.
   */
  call_json(function_: string, args_json: string): Promise<string>;
  /**
   * Runs a JS function against the element matching `selector` (`$eval`).
   */
  call_on_selector(selector: string, expression: string): Promise<string>;
  /**
   * Runs a JS function against every element matching `selector` (`$$eval`).
   */
  call_on_selector_all(selector: string, expression: string): Promise<string>;
  /**
   * Like [`Page::call_json`] but coerces the result to a string.
   */
  call_string(function_: string, args_json: string): Promise<string>;
  /**
   * Clears the recorded intercepted requests.
   */
  clear_requests(): Promise<void>;
  /**
   * Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
   */
  click_mouse(x: number, y: number): Promise<Page>;
  /**
   * Closes the page.
   */
  close(): Promise<void>;
  /**
   * Returns the full HTML content of the page.
   */
  content(): Promise<string>;
  /**
   * Returns a single cookie by name as JSON (or null).
   */
  cookie(name: string): Promise<string>;
  /**
   * Returns the cookies visible to this page as a JSON array.
   */
  cookies(): Promise<string>;
  /**
   * Starts CSS coverage collection.
   */
  coverage_start_css(): Promise<void>;
  /**
   * Starts JS coverage collection.
   */
  coverage_start_js(): Promise<void>;
  /**
   * Stops CSS coverage collection and returns the result as JSON.
   */
  coverage_stop_css(): Promise<string>;
  /**
   * Stops JS coverage collection and returns the result as JSON.
   */
  coverage_stop_js(): Promise<string>;
  /**
   * Returns the page PDF as a base64 string.
   */
  create_pdf_stream(): Promise<string>;
  decode_base64(data: string): Uint8Array;
  /**
   * Overrides the idle state.
   */
  emulate_idle_state(is_user_active: boolean, is_screen_unlocked: boolean): Promise<void>;
  /**
   * Emulates a media type and/or colour scheme.
   */
  emulate_media(media: string | undefined, color_scheme: string | undefined): Promise<void>;
  ensure_interception(): Promise<void>;
  /**
   * Evaluates JavaScript and coerces the result to a bool.
   */
  evaluate_bool(expression: string): Promise<boolean>;
  /**
   * Evaluates JavaScript and returns the resulting object as an [`Element`].
   */
  evaluate_handle(expression: string): Promise<Element>;
  /**
   * Evaluates JavaScript in the page and returns the result as a JSON string.
   */
  evaluate_json(expression: string): Promise<string>;
  /**
   * Evaluates JavaScript and coerces the result to a string.
   */
  evaluate_string(expression: string): Promise<string>;
  /**
   * Returns the registered event names.
   */
  event_names(): Promise<Array<string>>;
  /**
   * Escape hatch: sends an arbitrary CDP command and returns its JSON result.
   *
   * The adapters use this to express CDP-backed library methods as data
   * (a method name + parameter template), keeping the core surface small.
   */
  execute_cdp_cmd(method: string, params_json: string): Promise<string>;
  /**
   * Finds an element matching the CSS selector.
   */
  find_element(selector: string): Promise<Element>;
  /**
   * Returns the frame matching an id or name as JSON (or null).
   */
  frame(frame_id: string): Promise<string>;
  /**
   * Returns the main frame's name.
   */
  frame_name(): Promise<string>;
  /**
   * Returns every frame in the page as a JSON array.
   */
  frames(): Promise<string>;
  /**
   * Finds a form control by its associated `<label>` text.
   */
  get_by_label(label: string): Promise<Element>;
  /**
   * Finds an element by ARIA role (falls back to a tag-name lookup).
   */
  get_by_role(role: string): Promise<Element>;
  /**
   * Finds an element whose text content contains `text`.
   */
  get_by_text(text: string): Promise<Element>;
  /**
   * Returns the stored default timeout (ms).
   */
  get_default_timeout(): Promise<number>;
  go_back(): Promise<void>;
  /**
   * Navigates forward in history.
   */
  go_forward(): Promise<void>;
  /**
   * Accepts or dismisses the active JavaScript dialog.
   */
  handle_js_dialog(accept: boolean, prompt_text: string | undefined): Promise<void>;
  /**
   * Reads a local file and injects it as an init script.
   */
  inject_file(path: string): Promise<string>;
  /**
   * Whether drag interception is enabled.
   */
  is_drag_interception_enabled(): Promise<boolean>;
  /**
   * Dispatches a keydown event for `key`.
   */
  keyboard_down(key: string): Promise<void>;
  /**
   * Presses `key` on the focused element.
   */
  keyboard_press(key: string): Promise<void>;
  /**
   * Types `text` into the focused element.
   */
  keyboard_type(text: string): Promise<void>;
  /**
   * Dispatches a keyup event for `key`.
   */
  keyboard_up(key: string): Promise<void>;
  /**
   * Whether an event name is registered.
   */
  listens_to(event_name: string): Promise<boolean>;
  /**
   * Returns the main frame as JSON.
   */
  main_frame(): Promise<string>;
  /**
   * Returns the page performance metrics as a JSON object.
   */
  metrics(): Promise<string>;
  /**
   * Triggers a mousePress event at the current mouse coordinates.
   */
  mouse_down(button: string): Promise<Page>;
  /**
   * Triggers a mouseReleased event at the current mouse coordinates.
   */
  mouse_up(button: string): Promise<Page>;
  /**
   * Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
   */
  move_mouse(x: number, y: number): Promise<Page>;
  /**
   * Navigates to a URL.
   */
  navigate(url: string): Promise<void>;
  /**
   * Registers interest in a CDP event name.
   */
  on(event_name: string): Promise<void>;
  /**
   * Alias for [`Page::on`].
   */
  once(event_name: string): Promise<void>;
  pdf(): Promise<Uint8Array>;
  /**
   * Focuses the element matching `selector` and presses `key`.
   */
  press(selector: string, key: string): Promise<void>;
  /**
   * Returns every element matching the CSS selector.
   *
   * Uses two round trips (fetch the node list, then read its properties)
   * instead of one `evaluate` per match.
   */
  query_selector_all(selector: string): Promise<Array<Element>>;
  /**
   * Returns the first node matching an XPath expression as an [`Element`].
   */
  query_selector_xpath(xpath: string): Promise<Element>;
  raw_window_bounds(): Promise<string>;
  /**
   * Reloads the page.
   */
  reload(): Promise<void>;
  /**
   * Removes every registered event listener.
   */
  remove_all_listeners(): Promise<void>;
  /**
   * Removes a single registered event listener.
   */
  remove_listener(event_name: string): Promise<void>;
  /**
   * Removes an init script by identifier.
   */
  remove_script(identifier: string): Promise<void>;
  /**
   * Returns the most recent intercepted request as JSON.
   */
  request(): Promise<string>;
  /**
   * Returns the intercepted requests seen so far as JSON.
   */
  requests(): Promise<string>;
  /**
   * Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
   */
  route(pattern: string, action: string, body: string | undefined, content_type: string | undefined): Promise<void>;
  /**
   * Aborts every request matching `pattern`.
   */
  route_abort(pattern: string): Promise<void>;
  /**
   * Registers fulfill routes for every entry in a HAR file.
   */
  route_from_har(path: string): Promise<void>;
  /**
   * Fulfills every request matching `pattern` with `body`.
   */
  route_fulfill(pattern: string, body: string, content_type: string | undefined): Promise<void>;
  screenshot(): Promise<Uint8Array>;
  screenshot_full(): Promise<Uint8Array>;
  /**
   * Selects options by value/label on the matching `<select>`.
   */
  select_option(selector: string, values_json: string): Promise<void>;
  /**
   * Enables or disables the HTTP cache.
   */
  set_cache_enabled(enabled: boolean): Promise<void>;
  /**
   * Replaces the document content.
   */
  set_content(html: string): Promise<void>;
  /**
   * Stores a default timeout (ms) for adapter compatibility.
   */
  set_default_timeout(milliseconds: number): Promise<void>;
  /**
   * Enables or disables input drag interception.
   */
  set_drag_interception(enabled: boolean): Promise<void>;
  /**
   * Overrides media features (JSON array of `{name,value}`).
   */
  set_emulated_media_features(features_json: string): Promise<void>;
  /**
   * Sets extra HTTP headers for every request from this page.
   */
  set_extra_http_headers(headers_json: string): Promise<void>;
  /**
   * Sets the files of the matching `<input type="file">`.
   */
  set_input_files(selector: string, files_json: string): Promise<void>;
  /**
   * Enables or disables JavaScript execution.
   */
  set_javascript_enabled(enabled: boolean): Promise<void>;
  /**
   * Toggles offline mode.
   */
  set_offline(offline: boolean): Promise<void>;
  /**
   * Enables or disables request interception for this page.
   */
  set_request_interception(enabled: boolean): Promise<void>;
  /**
   * Restores cookies + localStorage from a storage-state JSON object.
   */
  set_storage_state(state_json: string): Promise<void>;
  /**
   * Overrides `navigator.userAgent` for this page.
   */
  set_user_agent(user_agent: string, accept_language: string | undefined): Promise<void>;
  /**
   * Overrides the viewport size.
   */
  set_viewport_size(width: bigint | number, height: bigint | number): Promise<void>;
  /**
   * Moves and resizes the window.
   */
  set_window_bounds(left: bigint | number, top: bigint | number, width: bigint | number, height: bigint | number): Promise<void>;
  /**
   * Moves the window, preserving its size.
   */
  set_window_position(x: bigint | number, y: bigint | number): Promise<void>;
  /**
   * Resizes the window, preserving its position.
   */
  set_window_size(width: bigint | number, height: bigint | number): Promise<void>;
  /**
   * Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
   */
  set_window_state(state: string): Promise<void>;
  /**
   * Starts a PNG screencast.
   */
  start_screencast(): Promise<void>;
  /**
   * Starts CDP tracing on this page's session.
   */
  start_tracing(): Promise<void>;
  /**
   * Stops the screencast.
   */
  stop_screencast(): Promise<void>;
  /**
   * Stops CDP tracing on this page's session.
   */
  stop_tracing(): Promise<void>;
  /**
   * Returns cookies + localStorage as a storage-state JSON object.
   */
  storage_state(): Promise<string>;
  /**
   * The CDP target id backing this page.
   */
  target_id(): string;
  /**
   * Returns the page title.
   */
  title(): Promise<string>;
  /**
   * Dispatches a touch tap at (x, y).
   */
  touch_tap(x: number, y: number): Promise<void>;
  /**
   * Removes the routes registered for `pattern`.
   */
  unroute(pattern: string): Promise<void>;
  /**
   * Removes every route rule.
   */
  unroute_all(): Promise<void>;
  /**
   * Returns the current document URL.
   */
  url(): Promise<string>;
  /**
   * Waits for the next CDP event named `event_name` and returns its params.
   *
   * The relevant domain is enabled first (best effort), so callers do not
   * have to.
   */
  wait_for_event(event_name: string, timeout_ms: bigint | number): Promise<string>;
  /**
   * [`Page::wait_for_event`] with the default 30s timeout.
   */
  wait_for_event_default(event_name: string): Promise<string>;
  /**
   * Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
   */
  wait_for_function(expression: string, timeout_ms: bigint | number): Promise<void>;
  /**
   * Waits for the page to finish loading.
   */
  wait_for_navigation(): Promise<void>;
  /**
   * Waits for an element matching the selector to appear in the DOM.
   */
  wait_for_selector(selector: string): Promise<Element>;
  /**
   * Waits for the first XPath match to appear.
   */
  wait_for_xpath(xpath: string, timeout_ms: bigint | number): Promise<Element>;
  window_id(): Promise<bigint | number>;
  /**
   * Returns the window position as JSON (`{x,y}`).
   */
  window_position(): Promise<string>;
  /**
   * Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
   */
  window_rect(): Promise<string>;
  /**
   * Returns the window size as JSON (`{width,height}`).
   */
  window_size(): Promise<string>;
}