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


export declare function uniffi_xcelerate_core_fn_clone_browser(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_free_browser(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_constructor_browser_launch(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_browser_close(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_browser_new_page(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_browser_version(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_clone_element(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_free_element(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_attribute(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_click(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_focus(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_hover(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_inner_html(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_text(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_element_type_text(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_clone_page(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_free_page(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_content(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_find_element(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_go_back(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_navigate(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_pdf(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_reload(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_screenshot_full(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_title(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_wait_for_navigation(...args: any[]): any;

export declare function uniffi_xcelerate_core_fn_method_page_wait_for_selector(...args: any[]): any;

export declare function ffi_xcelerate_core_rustbuffer_alloc(...args: any[]): any;

export declare function ffi_xcelerate_core_rustbuffer_from_bytes(...args: any[]): any;

export declare function ffi_xcelerate_core_rustbuffer_free(...args: any[]): any;

export declare function ffi_xcelerate_core_rustbuffer_reserve(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_u8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_u8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_u8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_u8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_i8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_i8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_i8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_i8(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_u16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_u16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_u16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_u16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_i16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_i16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_i16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_i16(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_u32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_u32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_u32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_u32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_i32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_i32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_i32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_i32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_u64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_u64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_u64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_u64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_i64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_i64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_i64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_i64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_f32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_f32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_f32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_f32(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_f64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_f64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_f64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_f64(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_rust_buffer(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_poll_void(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_cancel_void(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_free_void(...args: any[]): any;

export declare function ffi_xcelerate_core_rust_future_complete_void(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_browser_close(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_browser_new_page(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_browser_version(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_attribute(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_click(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_focus(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_hover(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_inner_html(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_text(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_element_type_text(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_content(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_find_element(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_go_back(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_navigate(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_pdf(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_reload(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_screenshot(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_screenshot_full(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_title(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_method_page_wait_for_selector(...args: any[]): any;

export declare function uniffi_xcelerate_core_checksum_constructor_browser_launch(...args: any[]): any;

export declare function ffi_xcelerate_core_uniffi_contract_version(...args: any[]): any;
