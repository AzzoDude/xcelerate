export interface FfiMetadata {
  namespace: string;
  cdylibName: string;
  stagedLibraryPackageRelativePath: string;
  bundledPrebuilds: boolean;
  manualLoad: boolean;
}

export interface FfiBindings {
  libraryPath: string;
  packageRelativePath: string | null;
  library: unknown;
  ffiTypes: Readonly<Record<string, unknown>>;
  ffiCallbacks: Readonly<Record<string, unknown>>;
  ffiStructs: Readonly<Record<string, unknown>>;
  ffiFunctions: Readonly<Record<string, (...args: any[]) => any>>;
}

export interface FfiIntegrity {
  contractVersionFunction: string;
  expectedContractVersion: number;
  checksums: Readonly<Record<string, number>>;
}

export interface FfiRuntimeHooks {
  onLoad?(bindings: Readonly<FfiBindings>): void;
  onUnload?(bindings: Readonly<FfiBindings>): void;
}

export declare const ffiMetadata: Readonly<FfiMetadata>;
export declare const ffiIntegrity: Readonly<FfiIntegrity>;
export declare function configureRuntimeHooks(hooks?: FfiRuntimeHooks | null): void;
export declare function load(libraryPath?: string | null): Readonly<FfiBindings>;
export declare function unload(): boolean;
export declare function isLoaded(): boolean;
export declare function getFfiBindings(): Readonly<FfiBindings>;
export declare function getFfiTypes(): Readonly<Record<string, unknown>>;
export declare function getContractVersion(bindings?: Readonly<FfiBindings>): number;
export declare function validateContractVersion(bindings?: Readonly<FfiBindings>): number;
export declare function getChecksums(
  bindings?: Readonly<FfiBindings>,
): Readonly<Record<string, number>>;
export declare function validateChecksums(
  bindings?: Readonly<FfiBindings>,
): Readonly<Record<string, number>>;
export declare const ffiFunctions: Readonly<Record<string, (...args: any[]) => any>>;


export declare function uniffi_xcelerate_fn_clone_browser(...args: any[]): any;

export declare function uniffi_xcelerate_fn_free_browser(...args: any[]): any;

export declare function uniffi_xcelerate_fn_constructor_browser_launch(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_browser_contexts(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_capabilities(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_close(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_cookies(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_delete_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_event_names(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_grant_permissions(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_is_connected(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_listens_to(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_new_context(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_new_page(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_on(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_once(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_remove_all_listeners(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_remove_listener(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_reset_permissions(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_set_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_set_download_behavior(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_start_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_stop_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_targets(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_user_agent(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_version(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_wait_for_event(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_wait_for_event_default(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_browser_ws_endpoint(...args: any[]): any;

export declare function uniffi_xcelerate_fn_clone_element(...args: any[]): any;

export declare function uniffi_xcelerate_fn_free_element(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_attribute(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_call_bool(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_call_json(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_call_on_selector(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_call_on_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_call_string(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_click(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_click_stealth(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_count(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_dispose(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_evaluate_bool(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_evaluate_handle(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_evaluate_json(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_evaluate_string(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_focus(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_get_by_label(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_get_by_role(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_get_by_text(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_get_properties(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_hover(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_hover_stealth(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_inner_html(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_press(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_query_selector(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_query_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_query_selector_attr(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_query_selector_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_screenshot_base64(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_select_option(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_set_input_files(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_text(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_type_text(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_element_wait_for_selector(...args: any[]): any;

export declare function uniffi_xcelerate_fn_clone_page(...args: any[]): any;

export declare function uniffi_xcelerate_fn_free_page(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_activate(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_activate_target(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_add_style_tag(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_authenticate(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_bring_to_front(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_call_bool(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_call_json(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_call_on_selector(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_call_on_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_call_string(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_clear_requests(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_click_mouse(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_close(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_content(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_cookies(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_coverage_start_css(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_coverage_start_js(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_coverage_stop_css(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_coverage_stop_js(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_create_pdf_stream(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_decode_base64(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_emulate_idle_state(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_emulate_media(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_ensure_interception(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_evaluate_bool(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_evaluate_handle(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_evaluate_json(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_evaluate_string(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_event_names(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_execute_cdp_cmd(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_find_element(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_frame(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_frame_name(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_frames(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_get_by_label(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_get_by_role(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_get_by_text(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_get_default_timeout(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_go_back(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_go_forward(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_handle_js_dialog(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_inject_file(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_keyboard_down(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_keyboard_press(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_keyboard_type(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_keyboard_up(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_listens_to(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_main_frame(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_metrics(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_mouse_down(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_mouse_up(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_move_mouse(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_navigate(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_on(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_once(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_pdf(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_press(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_query_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_query_selector_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_raw_window_bounds(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_reload(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_remove_all_listeners(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_remove_listener(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_remove_script(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_request(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_requests(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_route(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_route_abort(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_route_from_har(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_route_fulfill(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_screenshot_full(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_select_option(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_cache_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_content(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_default_timeout(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_drag_interception(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_emulated_media_features(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_extra_http_headers(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_input_files(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_javascript_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_offline(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_request_interception(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_storage_state(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_user_agent(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_viewport_size(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_window_bounds(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_window_position(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_window_size(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_set_window_state(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_start_screencast(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_start_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_stop_screencast(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_stop_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_storage_state(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_target_id(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_title(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_touch_tap(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_unroute(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_unroute_all(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_url(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_event(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_event_default(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_function(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_navigation(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_selector(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_wait_for_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_window_id(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_window_position(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_window_rect(...args: any[]): any;

export declare function uniffi_xcelerate_fn_method_page_window_size(...args: any[]): any;

export declare function ffi_xcelerate_rustbuffer_alloc(...args: any[]): any;

export declare function ffi_xcelerate_rustbuffer_from_bytes(...args: any[]): any;

export declare function ffi_xcelerate_rustbuffer_free(...args: any[]): any;

export declare function ffi_xcelerate_rustbuffer_reserve(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_u8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_u8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_u8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_u8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_i8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_i8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_i8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_i8(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_u16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_u16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_u16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_u16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_i16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_i16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_i16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_i16(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_u32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_u32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_u32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_u32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_i32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_i32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_i32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_i32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_u64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_u64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_u64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_u64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_i64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_i64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_i64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_i64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_f32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_f32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_f32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_f32(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_f64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_f64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_f64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_f64(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_poll_void(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_cancel_void(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_free_void(...args: any[]): any;

export declare function ffi_xcelerate_rust_future_complete_void(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_browser_contexts(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_capabilities(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_close(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_cookies(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_delete_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_event_names(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_grant_permissions(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_is_connected(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_listens_to(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_new_context(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_new_page(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_on(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_once(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_remove_all_listeners(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_remove_listener(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_reset_permissions(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_set_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_set_download_behavior(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_start_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_stop_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_targets(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_user_agent(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_version(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_wait_for_event(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_wait_for_event_default(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_browser_ws_endpoint(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_attribute(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_call_bool(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_call_json(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_call_on_selector(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_call_on_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_call_string(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_click(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_click_stealth(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_count(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_dispose(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_evaluate_bool(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_evaluate_handle(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_evaluate_json(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_evaluate_string(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_focus(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_get_by_label(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_get_by_role(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_get_by_text(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_get_properties(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_hover(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_hover_stealth(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_inner_html(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_press(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_query_selector(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_query_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_query_selector_attr(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_query_selector_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_screenshot_base64(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_select_option(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_set_input_files(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_text(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_type_text(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_element_wait_for_selector(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_activate(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_activate_target(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_add_style_tag(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_authenticate(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_bring_to_front(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_call_bool(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_call_json(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_call_on_selector(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_call_on_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_call_string(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_clear_requests(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_click_mouse(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_close(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_content(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_cookie(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_cookies(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_coverage_start_css(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_coverage_start_js(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_coverage_stop_css(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_coverage_stop_js(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_create_pdf_stream(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_decode_base64(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_emulate_idle_state(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_emulate_media(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_ensure_interception(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_evaluate_bool(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_evaluate_handle(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_evaluate_json(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_evaluate_string(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_event_names(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_find_element(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_frame(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_frame_name(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_frames(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_get_by_label(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_get_by_role(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_get_by_text(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_get_default_timeout(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_go_back(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_go_forward(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_handle_js_dialog(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_inject_file(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_keyboard_down(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_keyboard_press(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_keyboard_type(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_keyboard_up(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_listens_to(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_main_frame(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_metrics(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_mouse_down(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_mouse_up(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_move_mouse(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_navigate(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_on(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_once(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_pdf(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_press(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_query_selector_all(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_query_selector_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_raw_window_bounds(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_reload(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_remove_all_listeners(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_remove_listener(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_remove_script(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_request(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_requests(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_route(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_route_abort(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_route_from_har(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_route_fulfill(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_screenshot_full(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_select_option(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_cache_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_content(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_default_timeout(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_drag_interception(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_emulated_media_features(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_extra_http_headers(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_input_files(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_javascript_enabled(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_offline(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_request_interception(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_storage_state(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_user_agent(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_viewport_size(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_window_bounds(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_window_position(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_window_size(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_set_window_state(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_start_screencast(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_start_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_stop_screencast(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_stop_tracing(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_storage_state(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_target_id(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_title(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_touch_tap(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_unroute(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_unroute_all(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_url(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_event(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_event_default(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_function(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_navigation(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_selector(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_wait_for_xpath(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_window_id(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_window_position(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_window_rect(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_method_page_window_size(...args: any[]): any;

export declare function uniffi_xcelerate_checksum_constructor_browser_launch(...args: any[]): any;

export declare function ffi_xcelerate_uniffi_contract_version(...args: any[]): any;
