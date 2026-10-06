import { existsSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";
import koffi from "koffi";
import {
  ForeignBytes,
  RustBuffer,
  RustCallStatus,
  defineCallbackPrototype,
  UniffiHandle,
  VoidPointer,
  defineCallbackVtable,
  defineStructType,
  normalizeHandle,
  normalizeInt64,
  normalizeRustBuffer,
  normalizeRustCallStatus,
  normalizeUInt64,
} from "./runtime/ffi-types.js";
import {
  ChecksumMismatchError,
  ContractVersionMismatchError,
  LibraryNotLoadedError,
} from "./runtime/errors.js";

export const ffiMetadata = Object.freeze({
  namespace: "xcelerate",
  cdylibName: "xcelerate",
  stagedLibraryPackageRelativePath: "xcelerate.dll",
  bundledPrebuilds: false,
  manualLoad: false,
});

export const ffiIntegrity = Object.freeze({
  contractVersionFunction: "ffi_xcelerate_uniffi_contract_version",
  expectedContractVersion: 30,
  checksums: Object.freeze({

    "uniffi_xcelerate_checksum_method_browser_audit_log": 58417,

    "uniffi_xcelerate_checksum_method_browser_audit_verify": 56834,

    "uniffi_xcelerate_checksum_method_browser_available_plugins": 13132,

    "uniffi_xcelerate_checksum_method_browser_browser_contexts": 59339,

    "uniffi_xcelerate_checksum_method_browser_capabilities": 7601,

    "uniffi_xcelerate_checksum_method_browser_close": 44553,

    "uniffi_xcelerate_checksum_method_browser_cookies": 8530,

    "uniffi_xcelerate_checksum_method_browser_delete_cookie": 44579,

    "uniffi_xcelerate_checksum_method_browser_event_names": 12570,

    "uniffi_xcelerate_checksum_method_browser_grant_permissions": 62168,

    "uniffi_xcelerate_checksum_method_browser_is_connected": 63934,

    "uniffi_xcelerate_checksum_method_browser_listens_to": 56113,

    "uniffi_xcelerate_checksum_method_browser_load_plugin": 26734,

    "uniffi_xcelerate_checksum_method_browser_new_context": 59309,

    "uniffi_xcelerate_checksum_method_browser_new_page": 65142,

    "uniffi_xcelerate_checksum_method_browser_on": 4402,

    "uniffi_xcelerate_checksum_method_browser_once": 62023,

    "uniffi_xcelerate_checksum_method_browser_plugin": 38553,

    "uniffi_xcelerate_checksum_method_browser_plugin_names": 5716,

    "uniffi_xcelerate_checksum_method_browser_remove_all_listeners": 16672,

    "uniffi_xcelerate_checksum_method_browser_remove_listener": 3101,

    "uniffi_xcelerate_checksum_method_browser_reset_permissions": 50876,

    "uniffi_xcelerate_checksum_method_browser_set_cookie": 61323,

    "uniffi_xcelerate_checksum_method_browser_set_download_behavior": 32642,

    "uniffi_xcelerate_checksum_method_browser_start_tracing": 25818,

    "uniffi_xcelerate_checksum_method_browser_stop_tracing": 30224,

    "uniffi_xcelerate_checksum_method_browser_targets": 50695,

    "uniffi_xcelerate_checksum_method_browser_use_plugin": 53288,

    "uniffi_xcelerate_checksum_method_browser_user_agent": 36639,

    "uniffi_xcelerate_checksum_method_browser_version": 2891,

    "uniffi_xcelerate_checksum_method_browser_wait_for_event": 63537,

    "uniffi_xcelerate_checksum_method_browser_wait_for_event_default": 14198,

    "uniffi_xcelerate_checksum_method_browser_ws_endpoint": 63756,

    "uniffi_xcelerate_checksum_method_element_attribute": 8836,

    "uniffi_xcelerate_checksum_method_element_call_bool": 19329,

    "uniffi_xcelerate_checksum_method_element_call_json": 56720,

    "uniffi_xcelerate_checksum_method_element_call_on_selector": 53984,

    "uniffi_xcelerate_checksum_method_element_call_on_selector_all": 47977,

    "uniffi_xcelerate_checksum_method_element_call_string": 1191,

    "uniffi_xcelerate_checksum_method_element_click": 26136,

    "uniffi_xcelerate_checksum_method_element_click_mouse": 60796,

    "uniffi_xcelerate_checksum_method_element_count": 40137,

    "uniffi_xcelerate_checksum_method_element_dispose": 27134,

    "uniffi_xcelerate_checksum_method_element_evaluate_bool": 62708,

    "uniffi_xcelerate_checksum_method_element_evaluate_handle": 34826,

    "uniffi_xcelerate_checksum_method_element_evaluate_json": 20134,

    "uniffi_xcelerate_checksum_method_element_evaluate_string": 15210,

    "uniffi_xcelerate_checksum_method_element_focus": 34225,

    "uniffi_xcelerate_checksum_method_element_get_by_label": 29865,

    "uniffi_xcelerate_checksum_method_element_get_by_role": 15953,

    "uniffi_xcelerate_checksum_method_element_get_by_text": 18847,

    "uniffi_xcelerate_checksum_method_element_get_properties": 28646,

    "uniffi_xcelerate_checksum_method_element_hover": 32638,

    "uniffi_xcelerate_checksum_method_element_hover_mouse": 47391,

    "uniffi_xcelerate_checksum_method_element_inner_html": 63319,

    "uniffi_xcelerate_checksum_method_element_press": 13244,

    "uniffi_xcelerate_checksum_method_element_query_selector": 19454,

    "uniffi_xcelerate_checksum_method_element_query_selector_all": 65463,

    "uniffi_xcelerate_checksum_method_element_query_selector_attr": 63681,

    "uniffi_xcelerate_checksum_method_element_query_selector_xpath": 11390,

    "uniffi_xcelerate_checksum_method_element_screenshot": 55082,

    "uniffi_xcelerate_checksum_method_element_screenshot_base64": 62387,

    "uniffi_xcelerate_checksum_method_element_select_option": 21736,

    "uniffi_xcelerate_checksum_method_element_set_input_files": 32784,

    "uniffi_xcelerate_checksum_method_element_text": 41314,

    "uniffi_xcelerate_checksum_method_element_type_text": 45944,

    "uniffi_xcelerate_checksum_method_element_wait_for_selector": 23551,

    "uniffi_xcelerate_checksum_method_page_activate": 30852,

    "uniffi_xcelerate_checksum_method_page_activate_target": 6362,

    "uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document": 20123,

    "uniffi_xcelerate_checksum_method_page_add_style_tag": 9947,

    "uniffi_xcelerate_checksum_method_page_authenticate": 15669,

    "uniffi_xcelerate_checksum_method_page_bring_to_front": 14186,

    "uniffi_xcelerate_checksum_method_page_call_bool": 62656,

    "uniffi_xcelerate_checksum_method_page_call_json": 37033,

    "uniffi_xcelerate_checksum_method_page_call_on_selector": 902,

    "uniffi_xcelerate_checksum_method_page_call_on_selector_all": 9317,

    "uniffi_xcelerate_checksum_method_page_call_string": 28160,

    "uniffi_xcelerate_checksum_method_page_clear_requests": 25306,

    "uniffi_xcelerate_checksum_method_page_click_mouse": 54243,

    "uniffi_xcelerate_checksum_method_page_close": 53159,

    "uniffi_xcelerate_checksum_method_page_content": 15096,

    "uniffi_xcelerate_checksum_method_page_cookie": 45979,

    "uniffi_xcelerate_checksum_method_page_cookies": 45327,

    "uniffi_xcelerate_checksum_method_page_coverage_start_css": 860,

    "uniffi_xcelerate_checksum_method_page_coverage_start_js": 4186,

    "uniffi_xcelerate_checksum_method_page_coverage_stop_css": 59476,

    "uniffi_xcelerate_checksum_method_page_coverage_stop_js": 12341,

    "uniffi_xcelerate_checksum_method_page_create_pdf_stream": 54525,

    "uniffi_xcelerate_checksum_method_page_decode_base64": 39526,

    "uniffi_xcelerate_checksum_method_page_default_timeout": 18710,

    "uniffi_xcelerate_checksum_method_page_document_element": 41358,

    "uniffi_xcelerate_checksum_method_page_emulate_idle_state": 53017,

    "uniffi_xcelerate_checksum_method_page_emulate_media": 27664,

    "uniffi_xcelerate_checksum_method_page_ensure_interception": 6857,

    "uniffi_xcelerate_checksum_method_page_evaluate_bool": 12902,

    "uniffi_xcelerate_checksum_method_page_evaluate_handle": 57739,

    "uniffi_xcelerate_checksum_method_page_evaluate_json": 10653,

    "uniffi_xcelerate_checksum_method_page_evaluate_string": 6817,

    "uniffi_xcelerate_checksum_method_page_event_names": 15640,

    "uniffi_xcelerate_checksum_method_page_execute_cdp_cmd": 20070,

    "uniffi_xcelerate_checksum_method_page_find_element": 20082,

    "uniffi_xcelerate_checksum_method_page_frame": 33986,

    "uniffi_xcelerate_checksum_method_page_frame_name": 1668,

    "uniffi_xcelerate_checksum_method_page_frames": 13809,

    "uniffi_xcelerate_checksum_method_page_get_by_label": 51936,

    "uniffi_xcelerate_checksum_method_page_get_by_role": 32000,

    "uniffi_xcelerate_checksum_method_page_get_by_text": 25448,

    "uniffi_xcelerate_checksum_method_page_get_default_timeout": 57791,

    "uniffi_xcelerate_checksum_method_page_go_back": 60849,

    "uniffi_xcelerate_checksum_method_page_go_forward": 725,

    "uniffi_xcelerate_checksum_method_page_handle_js_dialog": 6178,

    "uniffi_xcelerate_checksum_method_page_inject_file": 16195,

    "uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled": 59945,

    "uniffi_xcelerate_checksum_method_page_keyboard_down": 53133,

    "uniffi_xcelerate_checksum_method_page_keyboard_press": 43593,

    "uniffi_xcelerate_checksum_method_page_keyboard_type": 53179,

    "uniffi_xcelerate_checksum_method_page_keyboard_up": 24408,

    "uniffi_xcelerate_checksum_method_page_listens_to": 42886,

    "uniffi_xcelerate_checksum_method_page_main_frame": 10285,

    "uniffi_xcelerate_checksum_method_page_metrics": 16960,

    "uniffi_xcelerate_checksum_method_page_mouse_down": 52368,

    "uniffi_xcelerate_checksum_method_page_mouse_up": 52299,

    "uniffi_xcelerate_checksum_method_page_move_mouse": 17586,

    "uniffi_xcelerate_checksum_method_page_navigate": 51495,

    "uniffi_xcelerate_checksum_method_page_on": 15602,

    "uniffi_xcelerate_checksum_method_page_once": 33401,

    "uniffi_xcelerate_checksum_method_page_pdf": 50825,

    "uniffi_xcelerate_checksum_method_page_press": 21741,

    "uniffi_xcelerate_checksum_method_page_query_selector_all": 7778,

    "uniffi_xcelerate_checksum_method_page_query_selector_xpath": 64720,

    "uniffi_xcelerate_checksum_method_page_raw_window_bounds": 13012,

    "uniffi_xcelerate_checksum_method_page_reload": 20867,

    "uniffi_xcelerate_checksum_method_page_remove_all_listeners": 31887,

    "uniffi_xcelerate_checksum_method_page_remove_listener": 35484,

    "uniffi_xcelerate_checksum_method_page_remove_script": 13802,

    "uniffi_xcelerate_checksum_method_page_request": 58954,

    "uniffi_xcelerate_checksum_method_page_requests": 29432,

    "uniffi_xcelerate_checksum_method_page_route": 24223,

    "uniffi_xcelerate_checksum_method_page_route_abort": 32588,

    "uniffi_xcelerate_checksum_method_page_route_from_har": 9232,

    "uniffi_xcelerate_checksum_method_page_route_fulfill": 29300,

    "uniffi_xcelerate_checksum_method_page_screenshot": 62867,

    "uniffi_xcelerate_checksum_method_page_screenshot_full": 56180,

    "uniffi_xcelerate_checksum_method_page_select_option": 5310,

    "uniffi_xcelerate_checksum_method_page_set_cache_enabled": 36286,

    "uniffi_xcelerate_checksum_method_page_set_content": 60133,

    "uniffi_xcelerate_checksum_method_page_set_default_timeout": 7299,

    "uniffi_xcelerate_checksum_method_page_set_drag_interception": 35102,

    "uniffi_xcelerate_checksum_method_page_set_emulated_media_features": 49723,

    "uniffi_xcelerate_checksum_method_page_set_extra_http_headers": 43902,

    "uniffi_xcelerate_checksum_method_page_set_input_files": 1582,

    "uniffi_xcelerate_checksum_method_page_set_javascript_enabled": 56309,

    "uniffi_xcelerate_checksum_method_page_set_offline": 37394,

    "uniffi_xcelerate_checksum_method_page_set_request_interception": 47016,

    "uniffi_xcelerate_checksum_method_page_set_storage_state": 55671,

    "uniffi_xcelerate_checksum_method_page_set_user_agent": 65506,

    "uniffi_xcelerate_checksum_method_page_set_viewport_size": 47964,

    "uniffi_xcelerate_checksum_method_page_set_window_bounds": 34825,

    "uniffi_xcelerate_checksum_method_page_set_window_position": 55844,

    "uniffi_xcelerate_checksum_method_page_set_window_size": 39920,

    "uniffi_xcelerate_checksum_method_page_set_window_state": 29849,

    "uniffi_xcelerate_checksum_method_page_start_screencast": 3179,

    "uniffi_xcelerate_checksum_method_page_start_tracing": 44969,

    "uniffi_xcelerate_checksum_method_page_stop_screencast": 14965,

    "uniffi_xcelerate_checksum_method_page_stop_tracing": 507,

    "uniffi_xcelerate_checksum_method_page_storage_state": 8033,

    "uniffi_xcelerate_checksum_method_page_target_id": 16602,

    "uniffi_xcelerate_checksum_method_page_title": 57758,

    "uniffi_xcelerate_checksum_method_page_touch_tap": 50285,

    "uniffi_xcelerate_checksum_method_page_unroute": 49265,

    "uniffi_xcelerate_checksum_method_page_unroute_all": 21098,

    "uniffi_xcelerate_checksum_method_page_url": 13992,

    "uniffi_xcelerate_checksum_method_page_wait_for_event": 16279,

    "uniffi_xcelerate_checksum_method_page_wait_for_event_default": 7458,

    "uniffi_xcelerate_checksum_method_page_wait_for_function": 39925,

    "uniffi_xcelerate_checksum_method_page_wait_for_navigation": 28813,

    "uniffi_xcelerate_checksum_method_page_wait_for_selector": 8076,

    "uniffi_xcelerate_checksum_method_page_wait_for_xpath": 14726,

    "uniffi_xcelerate_checksum_method_page_window_id": 52015,

    "uniffi_xcelerate_checksum_method_page_window_position": 23821,

    "uniffi_xcelerate_checksum_method_page_window_rect": 55898,

    "uniffi_xcelerate_checksum_method_page_window_size": 35222,

    "uniffi_xcelerate_checksum_method_pluginhandle_invoke": 29371,

    "uniffi_xcelerate_checksum_method_pluginhandle_ops": 11713,

    "uniffi_xcelerate_checksum_method_pluginhandle_plugin_name": 61258,

    "uniffi_xcelerate_checksum_constructor_browser_launch": 47265,

  }),
});

let loadedBindings = null;
let loadedFfiTypes = null;
let loadedFfiFunctions = null;
// Koffi retains native state for repeated lib.func() declarations, so keep a
// single binding core alive across unload/load cycles and evict stale cores
// when switching to a different canonical library path.
let cachedBindingCore = null;
let cachedLibraryPath = null;
let runtimeHooks = Object.freeze({});
const moduleFilename = fileURLToPath(import.meta.url);
const moduleDirectory = dirname(moduleFilename);
const libraryNotLoadedMessage =
  "The native library is not loaded. Call load(libraryPath) first.";

function bundledPrebuildPlatform() {
  switch (process.platform) {
    case "aix":
    case "android":
    case "darwin":
    case "freebsd":
    case "linux":
    case "openbsd":
    case "win32":
      return process.platform;
    default:
      throw new Error(
        `Unsupported Node platform ${JSON.stringify(process.platform)} for UniFFI bundled prebuild resolution.`,
      );
  }
}

function bundledPrebuildArch() {
  switch (process.arch) {
    case "arm":
    case "arm64":
    case "ia32":
    case "loong64":
    case "ppc64":
    case "riscv64":
    case "s390x":
    case "x64":
      return process.arch;
    default:
      throw new Error(
        `Unsupported Node architecture ${JSON.stringify(process.arch)} for UniFFI bundled prebuild resolution.`,
      );
  }
}

function defaultBundledTarget() {
  const platform = bundledPrebuildPlatform();
  const arch = bundledPrebuildArch();
  if (platform !== "linux") {
    return `${platform}-${arch}`;
  }

  const glibcVersionRuntime =
    process.report?.getReport?.().header?.glibcVersionRuntime;
  const linuxLibc = glibcVersionRuntime == null ? "musl" : "gnu";
  return `${platform}-${arch}-${linuxLibc}`;
}

function bundledLibraryFileName(platform) {
  switch (platform) {
    case "win32":
      return `${ffiMetadata.cdylibName}.dll`;
    case "darwin":
      return `lib${ffiMetadata.cdylibName}.dylib`;
    case "aix":
    case "android":
    case "freebsd":
    case "linux":
    case "openbsd":
      return `lib${ffiMetadata.cdylibName}.so`;
    default:
      throw new Error(
        `Unsupported Node platform ${JSON.stringify(platform)} for UniFFI bundled prebuild resolution.`,
      );
  }
}

function defaultBundledLibrary() {
  const platform = bundledPrebuildPlatform();
  const target = defaultBundledTarget();
  const filename = bundledLibraryFileName(platform);
  return Object.freeze({
    target,
    packageRelativePath: `prebuilds/${target}/${filename}`,
    libraryPath: join(moduleDirectory, "prebuilds", target, filename),
  });
}

function defaultSiblingLibraryPath() {
  return join(moduleDirectory, ffiMetadata.stagedLibraryPackageRelativePath);
}

function resolveLibraryPath(libraryPath = undefined) {
  if (libraryPath != null) {
    return Object.freeze({
      libraryPath: isAbsolute(libraryPath)
        ? libraryPath
        : join(moduleDirectory, libraryPath),
      packageRelativePath: null,
      bundledPrebuild: null,
    });
  }

  if (ffiMetadata.bundledPrebuilds) {
    const bundledPrebuild = defaultBundledLibrary();
    return Object.freeze({
      libraryPath: bundledPrebuild.libraryPath,
      packageRelativePath: bundledPrebuild.packageRelativePath,
      bundledPrebuild,
    });
  }

  return Object.freeze({
    libraryPath: defaultSiblingLibraryPath(),
    packageRelativePath: ffiMetadata.stagedLibraryPackageRelativePath,
    bundledPrebuild: null,
  });
}

function canonicalizeExistingLibraryPath(libraryPath) {
  if (!existsSync(libraryPath)) {
    return libraryPath;
  }

  return typeof realpathSync.native === "function"
    ? realpathSync.native(libraryPath)
    : realpathSync(libraryPath);
}

function createBindingCore(libraryPath) {
  const library = koffi.load(libraryPath);

  const ffiTypes = Object.freeze({
    UniffiHandle,
    VoidPointer,
    RustBuffer,
    ForeignBytes,
    RustCallStatus,
  });
  const ffiCallbacks = {};

  ffiCallbacks.RustFutureContinuationCallback = defineCallbackPrototype("RustFutureContinuationCallback", "void", ["uint64_t", "int8_t"]);

  ffiCallbacks.ForeignFutureDroppedCallback = defineCallbackPrototype("ForeignFutureDroppedCallback", "void", ["uint64_t"]);

  ffiCallbacks.CallbackInterfaceFree = defineCallbackPrototype("CallbackInterfaceFree", "void", ["uint64_t"]);

  ffiCallbacks.CallbackInterfaceClone = defineCallbackPrototype("CallbackInterfaceClone", "uint64_t", ["uint64_t"]);

  const ffiStructs = {};


  ffiStructs.ForeignFutureDroppedCallbackStruct = defineStructType("ForeignFutureDroppedCallbackStruct", {

      "handle": "uint64_t",

      "free": koffi.pointer(ffiCallbacks.ForeignFutureDroppedCallback),

  });



  ffiStructs.ForeignFutureResultU8 = defineStructType("ForeignFutureResultU8", {

      "return_value": "uint8_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultI8 = defineStructType("ForeignFutureResultI8", {

      "return_value": "int8_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultU16 = defineStructType("ForeignFutureResultU16", {

      "return_value": "uint16_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultI16 = defineStructType("ForeignFutureResultI16", {

      "return_value": "int16_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultU32 = defineStructType("ForeignFutureResultU32", {

      "return_value": "uint32_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultI32 = defineStructType("ForeignFutureResultI32", {

      "return_value": "int32_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultU64 = defineStructType("ForeignFutureResultU64", {

      "return_value": "uint64_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultI64 = defineStructType("ForeignFutureResultI64", {

      "return_value": "int64_t",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultF32 = defineStructType("ForeignFutureResultF32", {

      "return_value": "float",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultF64 = defineStructType("ForeignFutureResultF64", {

      "return_value": "double",

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultRustBuffer = defineStructType("ForeignFutureResultRustBuffer", {

      "return_value": ffiTypes.RustBuffer,

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiStructs.ForeignFutureResultVoid = defineStructType("ForeignFutureResultVoid", {

      "call_status": ffiTypes.RustCallStatus,

  });



  ffiCallbacks.ForeignFutureCompleteU8 = defineCallbackPrototype("ForeignFutureCompleteU8", "void", ["uint64_t", ffiStructs.ForeignFutureResultU8]);

  ffiCallbacks.ForeignFutureCompleteI8 = defineCallbackPrototype("ForeignFutureCompleteI8", "void", ["uint64_t", ffiStructs.ForeignFutureResultI8]);

  ffiCallbacks.ForeignFutureCompleteU16 = defineCallbackPrototype("ForeignFutureCompleteU16", "void", ["uint64_t", ffiStructs.ForeignFutureResultU16]);

  ffiCallbacks.ForeignFutureCompleteI16 = defineCallbackPrototype("ForeignFutureCompleteI16", "void", ["uint64_t", ffiStructs.ForeignFutureResultI16]);

  ffiCallbacks.ForeignFutureCompleteU32 = defineCallbackPrototype("ForeignFutureCompleteU32", "void", ["uint64_t", ffiStructs.ForeignFutureResultU32]);

  ffiCallbacks.ForeignFutureCompleteI32 = defineCallbackPrototype("ForeignFutureCompleteI32", "void", ["uint64_t", ffiStructs.ForeignFutureResultI32]);

  ffiCallbacks.ForeignFutureCompleteU64 = defineCallbackPrototype("ForeignFutureCompleteU64", "void", ["uint64_t", ffiStructs.ForeignFutureResultU64]);

  ffiCallbacks.ForeignFutureCompleteI64 = defineCallbackPrototype("ForeignFutureCompleteI64", "void", ["uint64_t", ffiStructs.ForeignFutureResultI64]);

  ffiCallbacks.ForeignFutureCompleteF32 = defineCallbackPrototype("ForeignFutureCompleteF32", "void", ["uint64_t", ffiStructs.ForeignFutureResultF32]);

  ffiCallbacks.ForeignFutureCompleteF64 = defineCallbackPrototype("ForeignFutureCompleteF64", "void", ["uint64_t", ffiStructs.ForeignFutureResultF64]);

  ffiCallbacks.ForeignFutureCompleteRustBuffer = defineCallbackPrototype("ForeignFutureCompleteRustBuffer", "void", ["uint64_t", ffiStructs.ForeignFutureResultRustBuffer]);

  ffiCallbacks.ForeignFutureCompleteVoid = defineCallbackPrototype("ForeignFutureCompleteVoid", "void", ["uint64_t", ffiStructs.ForeignFutureResultVoid]);




























  Object.freeze(ffiCallbacks);
  Object.freeze(ffiStructs);
  const ffiFunctions = Object.freeze({

    uniffi_xcelerate_fn_clone_browser: library.func("uniffi_xcelerate_fn_clone_browser", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_clone_browser_generic_abi: library.func("uniffi_xcelerate_fn_clone_browser", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_free_browser: library.func("uniffi_xcelerate_fn_free_browser", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_free_browser_generic_abi: library.func("uniffi_xcelerate_fn_free_browser", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_constructor_browser_launch: library.func("uniffi_xcelerate_fn_constructor_browser_launch", ffiTypes.UniffiHandle, [ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_audit_log: library.func("uniffi_xcelerate_fn_method_browser_audit_log", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_audit_log_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_audit_log", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_audit_verify: library.func("uniffi_xcelerate_fn_method_browser_audit_verify", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_audit_verify", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_available_plugins: library.func("uniffi_xcelerate_fn_method_browser_available_plugins", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_available_plugins", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_browser_contexts: library.func("uniffi_xcelerate_fn_method_browser_browser_contexts", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_browser_contexts", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_capabilities: library.func("uniffi_xcelerate_fn_method_browser_capabilities", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_capabilities_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_capabilities", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_close: library.func("uniffi_xcelerate_fn_method_browser_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_close_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_cookies: library.func("uniffi_xcelerate_fn_method_browser_cookies", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_cookies_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_cookies", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_delete_cookie: library.func("uniffi_xcelerate_fn_method_browser_delete_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_delete_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_event_names: library.func("uniffi_xcelerate_fn_method_browser_event_names", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_event_names_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_event_names", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_grant_permissions: library.func("uniffi_xcelerate_fn_method_browser_grant_permissions", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_grant_permissions", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_is_connected: library.func("uniffi_xcelerate_fn_method_browser_is_connected", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_is_connected_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_is_connected", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_listens_to: library.func("uniffi_xcelerate_fn_method_browser_listens_to", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_listens_to_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_listens_to", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_load_plugin: library.func("uniffi_xcelerate_fn_method_browser_load_plugin", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_load_plugin", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_new_context: library.func("uniffi_xcelerate_fn_method_browser_new_context", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_new_context_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_new_context", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_new_page: library.func("uniffi_xcelerate_fn_method_browser_new_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_new_page_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_new_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_on: library.func("uniffi_xcelerate_fn_method_browser_on", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_on_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_on", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_once: library.func("uniffi_xcelerate_fn_method_browser_once", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_once_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_once", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_plugin: library.func("uniffi_xcelerate_fn_method_browser_plugin", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_plugin_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_plugin", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_plugin_names: library.func("uniffi_xcelerate_fn_method_browser_plugin_names", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_plugin_names", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_browser_remove_all_listeners: library.func("uniffi_xcelerate_fn_method_browser_remove_all_listeners", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_remove_all_listeners", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_remove_listener: library.func("uniffi_xcelerate_fn_method_browser_remove_listener", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_remove_listener", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_reset_permissions: library.func("uniffi_xcelerate_fn_method_browser_reset_permissions", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_reset_permissions", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_set_cookie: library.func("uniffi_xcelerate_fn_method_browser_set_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_set_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_set_download_behavior: library.func("uniffi_xcelerate_fn_method_browser_set_download_behavior", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_set_download_behavior", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_start_tracing: library.func("uniffi_xcelerate_fn_method_browser_start_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_start_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_stop_tracing: library.func("uniffi_xcelerate_fn_method_browser_stop_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_stop_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_targets: library.func("uniffi_xcelerate_fn_method_browser_targets", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_targets_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_targets", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_use_plugin: library.func("uniffi_xcelerate_fn_method_browser_use_plugin", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_use_plugin", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_user_agent: library.func("uniffi_xcelerate_fn_method_browser_user_agent", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_user_agent_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_user_agent", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_version: library.func("uniffi_xcelerate_fn_method_browser_version", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_browser_version_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_version", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_browser_wait_for_event: library.func("uniffi_xcelerate_fn_method_browser_wait_for_event", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),

    uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_wait_for_event", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),


    uniffi_xcelerate_fn_method_browser_wait_for_event_default: library.func("uniffi_xcelerate_fn_method_browser_wait_for_event_default", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_wait_for_event_default", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_browser_ws_endpoint: library.func("uniffi_xcelerate_fn_method_browser_ws_endpoint", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi: library.func("uniffi_xcelerate_fn_method_browser_ws_endpoint", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_clone_element: library.func("uniffi_xcelerate_fn_clone_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_clone_element_generic_abi: library.func("uniffi_xcelerate_fn_clone_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_free_element: library.func("uniffi_xcelerate_fn_free_element", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_free_element_generic_abi: library.func("uniffi_xcelerate_fn_free_element", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_element_attribute: library.func("uniffi_xcelerate_fn_method_element_attribute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_attribute_generic_abi: library.func("uniffi_xcelerate_fn_method_element_attribute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_call_bool: library.func("uniffi_xcelerate_fn_method_element_call_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_call_bool_generic_abi: library.func("uniffi_xcelerate_fn_method_element_call_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_call_json: library.func("uniffi_xcelerate_fn_method_element_call_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_call_json_generic_abi: library.func("uniffi_xcelerate_fn_method_element_call_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_call_on_selector: library.func("uniffi_xcelerate_fn_method_element_call_on_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi: library.func("uniffi_xcelerate_fn_method_element_call_on_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_call_on_selector_all: library.func("uniffi_xcelerate_fn_method_element_call_on_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi: library.func("uniffi_xcelerate_fn_method_element_call_on_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_call_string: library.func("uniffi_xcelerate_fn_method_element_call_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_call_string_generic_abi: library.func("uniffi_xcelerate_fn_method_element_call_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_click: library.func("uniffi_xcelerate_fn_method_element_click", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_click_generic_abi: library.func("uniffi_xcelerate_fn_method_element_click", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_click_mouse: library.func("uniffi_xcelerate_fn_method_element_click_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_click_mouse_generic_abi: library.func("uniffi_xcelerate_fn_method_element_click_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_count: library.func("uniffi_xcelerate_fn_method_element_count", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_count_generic_abi: library.func("uniffi_xcelerate_fn_method_element_count", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_dispose: library.func("uniffi_xcelerate_fn_method_element_dispose", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_dispose_generic_abi: library.func("uniffi_xcelerate_fn_method_element_dispose", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_evaluate_bool: library.func("uniffi_xcelerate_fn_method_element_evaluate_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi: library.func("uniffi_xcelerate_fn_method_element_evaluate_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_evaluate_handle: library.func("uniffi_xcelerate_fn_method_element_evaluate_handle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi: library.func("uniffi_xcelerate_fn_method_element_evaluate_handle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_evaluate_json: library.func("uniffi_xcelerate_fn_method_element_evaluate_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi: library.func("uniffi_xcelerate_fn_method_element_evaluate_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_evaluate_string: library.func("uniffi_xcelerate_fn_method_element_evaluate_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi: library.func("uniffi_xcelerate_fn_method_element_evaluate_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_focus: library.func("uniffi_xcelerate_fn_method_element_focus", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_focus_generic_abi: library.func("uniffi_xcelerate_fn_method_element_focus", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_get_by_label: library.func("uniffi_xcelerate_fn_method_element_get_by_label", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_get_by_label_generic_abi: library.func("uniffi_xcelerate_fn_method_element_get_by_label", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_get_by_role: library.func("uniffi_xcelerate_fn_method_element_get_by_role", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_get_by_role_generic_abi: library.func("uniffi_xcelerate_fn_method_element_get_by_role", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_get_by_text: library.func("uniffi_xcelerate_fn_method_element_get_by_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_get_by_text_generic_abi: library.func("uniffi_xcelerate_fn_method_element_get_by_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_get_properties: library.func("uniffi_xcelerate_fn_method_element_get_properties", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_get_properties_generic_abi: library.func("uniffi_xcelerate_fn_method_element_get_properties", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_hover: library.func("uniffi_xcelerate_fn_method_element_hover", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_hover_generic_abi: library.func("uniffi_xcelerate_fn_method_element_hover", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_hover_mouse: library.func("uniffi_xcelerate_fn_method_element_hover_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_hover_mouse_generic_abi: library.func("uniffi_xcelerate_fn_method_element_hover_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_inner_html: library.func("uniffi_xcelerate_fn_method_element_inner_html", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_inner_html_generic_abi: library.func("uniffi_xcelerate_fn_method_element_inner_html", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_press: library.func("uniffi_xcelerate_fn_method_element_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_press_generic_abi: library.func("uniffi_xcelerate_fn_method_element_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_query_selector: library.func("uniffi_xcelerate_fn_method_element_query_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_query_selector_generic_abi: library.func("uniffi_xcelerate_fn_method_element_query_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_query_selector_all: library.func("uniffi_xcelerate_fn_method_element_query_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi: library.func("uniffi_xcelerate_fn_method_element_query_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_query_selector_attr: library.func("uniffi_xcelerate_fn_method_element_query_selector_attr", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi: library.func("uniffi_xcelerate_fn_method_element_query_selector_attr", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_query_selector_xpath: library.func("uniffi_xcelerate_fn_method_element_query_selector_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi: library.func("uniffi_xcelerate_fn_method_element_query_selector_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_screenshot: library.func("uniffi_xcelerate_fn_method_element_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_screenshot_generic_abi: library.func("uniffi_xcelerate_fn_method_element_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_screenshot_base64: library.func("uniffi_xcelerate_fn_method_element_screenshot_base64", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi: library.func("uniffi_xcelerate_fn_method_element_screenshot_base64", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_select_option: library.func("uniffi_xcelerate_fn_method_element_select_option", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_select_option_generic_abi: library.func("uniffi_xcelerate_fn_method_element_select_option", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_set_input_files: library.func("uniffi_xcelerate_fn_method_element_set_input_files", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_set_input_files_generic_abi: library.func("uniffi_xcelerate_fn_method_element_set_input_files", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_text: library.func("uniffi_xcelerate_fn_method_element_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_element_text_generic_abi: library.func("uniffi_xcelerate_fn_method_element_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_element_type_text: library.func("uniffi_xcelerate_fn_method_element_type_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_type_text_generic_abi: library.func("uniffi_xcelerate_fn_method_element_type_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_element_wait_for_selector: library.func("uniffi_xcelerate_fn_method_element_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi: library.func("uniffi_xcelerate_fn_method_element_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_clone_page: library.func("uniffi_xcelerate_fn_clone_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_clone_page_generic_abi: library.func("uniffi_xcelerate_fn_clone_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_free_page: library.func("uniffi_xcelerate_fn_free_page", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_free_page_generic_abi: library.func("uniffi_xcelerate_fn_free_page", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_page_activate: library.func("uniffi_xcelerate_fn_method_page_activate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_activate_generic_abi: library.func("uniffi_xcelerate_fn_method_page_activate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_activate_target: library.func("uniffi_xcelerate_fn_method_page_activate_target", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_activate_target_generic_abi: library.func("uniffi_xcelerate_fn_method_page_activate_target", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document: library.func("uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi: library.func("uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_add_style_tag: library.func("uniffi_xcelerate_fn_method_page_add_style_tag", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi: library.func("uniffi_xcelerate_fn_method_page_add_style_tag", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_authenticate: library.func("uniffi_xcelerate_fn_method_page_authenticate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_authenticate_generic_abi: library.func("uniffi_xcelerate_fn_method_page_authenticate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_bring_to_front: library.func("uniffi_xcelerate_fn_method_page_bring_to_front", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi: library.func("uniffi_xcelerate_fn_method_page_bring_to_front", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_call_bool: library.func("uniffi_xcelerate_fn_method_page_call_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_call_bool_generic_abi: library.func("uniffi_xcelerate_fn_method_page_call_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_call_json: library.func("uniffi_xcelerate_fn_method_page_call_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_call_json_generic_abi: library.func("uniffi_xcelerate_fn_method_page_call_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_call_on_selector: library.func("uniffi_xcelerate_fn_method_page_call_on_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi: library.func("uniffi_xcelerate_fn_method_page_call_on_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_call_on_selector_all: library.func("uniffi_xcelerate_fn_method_page_call_on_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi: library.func("uniffi_xcelerate_fn_method_page_call_on_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_call_string: library.func("uniffi_xcelerate_fn_method_page_call_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_call_string_generic_abi: library.func("uniffi_xcelerate_fn_method_page_call_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_clear_requests: library.func("uniffi_xcelerate_fn_method_page_clear_requests", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_clear_requests_generic_abi: library.func("uniffi_xcelerate_fn_method_page_clear_requests", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_click_mouse: library.func("uniffi_xcelerate_fn_method_page_click_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),

    uniffi_xcelerate_fn_method_page_click_mouse_generic_abi: library.func("uniffi_xcelerate_fn_method_page_click_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),


    uniffi_xcelerate_fn_method_page_close: library.func("uniffi_xcelerate_fn_method_page_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_close_generic_abi: library.func("uniffi_xcelerate_fn_method_page_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_content: library.func("uniffi_xcelerate_fn_method_page_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_content_generic_abi: library.func("uniffi_xcelerate_fn_method_page_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_cookie: library.func("uniffi_xcelerate_fn_method_page_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_cookie_generic_abi: library.func("uniffi_xcelerate_fn_method_page_cookie", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_cookies: library.func("uniffi_xcelerate_fn_method_page_cookies", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_cookies_generic_abi: library.func("uniffi_xcelerate_fn_method_page_cookies", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_coverage_start_css: library.func("uniffi_xcelerate_fn_method_page_coverage_start_css", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi: library.func("uniffi_xcelerate_fn_method_page_coverage_start_css", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_coverage_start_js: library.func("uniffi_xcelerate_fn_method_page_coverage_start_js", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi: library.func("uniffi_xcelerate_fn_method_page_coverage_start_js", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_coverage_stop_css: library.func("uniffi_xcelerate_fn_method_page_coverage_stop_css", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi: library.func("uniffi_xcelerate_fn_method_page_coverage_stop_css", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_coverage_stop_js: library.func("uniffi_xcelerate_fn_method_page_coverage_stop_js", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi: library.func("uniffi_xcelerate_fn_method_page_coverage_stop_js", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_create_pdf_stream: library.func("uniffi_xcelerate_fn_method_page_create_pdf_stream", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi: library.func("uniffi_xcelerate_fn_method_page_create_pdf_stream", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_decode_base64: library.func("uniffi_xcelerate_fn_method_page_decode_base64", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_page_decode_base64_generic_abi: library.func("uniffi_xcelerate_fn_method_page_decode_base64", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_page_default_timeout: library.func("uniffi_xcelerate_fn_method_page_default_timeout", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_page_default_timeout_generic_abi: library.func("uniffi_xcelerate_fn_method_page_default_timeout", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_page_document_element: library.func("uniffi_xcelerate_fn_method_page_document_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_document_element_generic_abi: library.func("uniffi_xcelerate_fn_method_page_document_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_emulate_idle_state: library.func("uniffi_xcelerate_fn_method_page_emulate_idle_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t", "int8_t"]),

    uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi: library.func("uniffi_xcelerate_fn_method_page_emulate_idle_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t", "int8_t"]),


    uniffi_xcelerate_fn_method_page_emulate_media: library.func("uniffi_xcelerate_fn_method_page_emulate_media", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_emulate_media_generic_abi: library.func("uniffi_xcelerate_fn_method_page_emulate_media", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_ensure_interception: library.func("uniffi_xcelerate_fn_method_page_ensure_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi: library.func("uniffi_xcelerate_fn_method_page_ensure_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_evaluate_bool: library.func("uniffi_xcelerate_fn_method_page_evaluate_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi: library.func("uniffi_xcelerate_fn_method_page_evaluate_bool", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_evaluate_handle: library.func("uniffi_xcelerate_fn_method_page_evaluate_handle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi: library.func("uniffi_xcelerate_fn_method_page_evaluate_handle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_evaluate_json: library.func("uniffi_xcelerate_fn_method_page_evaluate_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi: library.func("uniffi_xcelerate_fn_method_page_evaluate_json", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_evaluate_string: library.func("uniffi_xcelerate_fn_method_page_evaluate_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi: library.func("uniffi_xcelerate_fn_method_page_evaluate_string", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_event_names: library.func("uniffi_xcelerate_fn_method_page_event_names", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_event_names_generic_abi: library.func("uniffi_xcelerate_fn_method_page_event_names", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_execute_cdp_cmd: library.func("uniffi_xcelerate_fn_method_page_execute_cdp_cmd", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi: library.func("uniffi_xcelerate_fn_method_page_execute_cdp_cmd", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_find_element: library.func("uniffi_xcelerate_fn_method_page_find_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_find_element_generic_abi: library.func("uniffi_xcelerate_fn_method_page_find_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_frame: library.func("uniffi_xcelerate_fn_method_page_frame", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_frame_generic_abi: library.func("uniffi_xcelerate_fn_method_page_frame", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_frame_name: library.func("uniffi_xcelerate_fn_method_page_frame_name", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_frame_name_generic_abi: library.func("uniffi_xcelerate_fn_method_page_frame_name", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_frames: library.func("uniffi_xcelerate_fn_method_page_frames", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_frames_generic_abi: library.func("uniffi_xcelerate_fn_method_page_frames", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_get_by_label: library.func("uniffi_xcelerate_fn_method_page_get_by_label", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_get_by_label_generic_abi: library.func("uniffi_xcelerate_fn_method_page_get_by_label", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_get_by_role: library.func("uniffi_xcelerate_fn_method_page_get_by_role", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_get_by_role_generic_abi: library.func("uniffi_xcelerate_fn_method_page_get_by_role", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_get_by_text: library.func("uniffi_xcelerate_fn_method_page_get_by_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_get_by_text_generic_abi: library.func("uniffi_xcelerate_fn_method_page_get_by_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_get_default_timeout: library.func("uniffi_xcelerate_fn_method_page_get_default_timeout", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi: library.func("uniffi_xcelerate_fn_method_page_get_default_timeout", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_go_back: library.func("uniffi_xcelerate_fn_method_page_go_back", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_go_back_generic_abi: library.func("uniffi_xcelerate_fn_method_page_go_back", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_go_forward: library.func("uniffi_xcelerate_fn_method_page_go_forward", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_go_forward_generic_abi: library.func("uniffi_xcelerate_fn_method_page_go_forward", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_handle_js_dialog: library.func("uniffi_xcelerate_fn_method_page_handle_js_dialog", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t", ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi: library.func("uniffi_xcelerate_fn_method_page_handle_js_dialog", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t", ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_inject_file: library.func("uniffi_xcelerate_fn_method_page_inject_file", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_inject_file_generic_abi: library.func("uniffi_xcelerate_fn_method_page_inject_file", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_is_drag_interception_enabled: library.func("uniffi_xcelerate_fn_method_page_is_drag_interception_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi: library.func("uniffi_xcelerate_fn_method_page_is_drag_interception_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_keyboard_down: library.func("uniffi_xcelerate_fn_method_page_keyboard_down", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi: library.func("uniffi_xcelerate_fn_method_page_keyboard_down", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_keyboard_press: library.func("uniffi_xcelerate_fn_method_page_keyboard_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi: library.func("uniffi_xcelerate_fn_method_page_keyboard_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_keyboard_type: library.func("uniffi_xcelerate_fn_method_page_keyboard_type", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi: library.func("uniffi_xcelerate_fn_method_page_keyboard_type", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_keyboard_up: library.func("uniffi_xcelerate_fn_method_page_keyboard_up", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi: library.func("uniffi_xcelerate_fn_method_page_keyboard_up", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_listens_to: library.func("uniffi_xcelerate_fn_method_page_listens_to", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_listens_to_generic_abi: library.func("uniffi_xcelerate_fn_method_page_listens_to", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_main_frame: library.func("uniffi_xcelerate_fn_method_page_main_frame", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_main_frame_generic_abi: library.func("uniffi_xcelerate_fn_method_page_main_frame", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_metrics: library.func("uniffi_xcelerate_fn_method_page_metrics", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_metrics_generic_abi: library.func("uniffi_xcelerate_fn_method_page_metrics", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_mouse_down: library.func("uniffi_xcelerate_fn_method_page_mouse_down", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_mouse_down_generic_abi: library.func("uniffi_xcelerate_fn_method_page_mouse_down", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_mouse_up: library.func("uniffi_xcelerate_fn_method_page_mouse_up", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_mouse_up_generic_abi: library.func("uniffi_xcelerate_fn_method_page_mouse_up", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_move_mouse: library.func("uniffi_xcelerate_fn_method_page_move_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),

    uniffi_xcelerate_fn_method_page_move_mouse_generic_abi: library.func("uniffi_xcelerate_fn_method_page_move_mouse", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),


    uniffi_xcelerate_fn_method_page_navigate: library.func("uniffi_xcelerate_fn_method_page_navigate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_navigate_generic_abi: library.func("uniffi_xcelerate_fn_method_page_navigate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_on: library.func("uniffi_xcelerate_fn_method_page_on", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_on_generic_abi: library.func("uniffi_xcelerate_fn_method_page_on", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_once: library.func("uniffi_xcelerate_fn_method_page_once", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_once_generic_abi: library.func("uniffi_xcelerate_fn_method_page_once", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_pdf: library.func("uniffi_xcelerate_fn_method_page_pdf", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_pdf_generic_abi: library.func("uniffi_xcelerate_fn_method_page_pdf", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_press: library.func("uniffi_xcelerate_fn_method_page_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_press_generic_abi: library.func("uniffi_xcelerate_fn_method_page_press", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_query_selector_all: library.func("uniffi_xcelerate_fn_method_page_query_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi: library.func("uniffi_xcelerate_fn_method_page_query_selector_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_query_selector_xpath: library.func("uniffi_xcelerate_fn_method_page_query_selector_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi: library.func("uniffi_xcelerate_fn_method_page_query_selector_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_raw_window_bounds: library.func("uniffi_xcelerate_fn_method_page_raw_window_bounds", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi: library.func("uniffi_xcelerate_fn_method_page_raw_window_bounds", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_reload: library.func("uniffi_xcelerate_fn_method_page_reload", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_reload_generic_abi: library.func("uniffi_xcelerate_fn_method_page_reload", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_remove_all_listeners: library.func("uniffi_xcelerate_fn_method_page_remove_all_listeners", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi: library.func("uniffi_xcelerate_fn_method_page_remove_all_listeners", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_remove_listener: library.func("uniffi_xcelerate_fn_method_page_remove_listener", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_remove_listener_generic_abi: library.func("uniffi_xcelerate_fn_method_page_remove_listener", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_remove_script: library.func("uniffi_xcelerate_fn_method_page_remove_script", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_remove_script_generic_abi: library.func("uniffi_xcelerate_fn_method_page_remove_script", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_request: library.func("uniffi_xcelerate_fn_method_page_request", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_request_generic_abi: library.func("uniffi_xcelerate_fn_method_page_request", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_requests: library.func("uniffi_xcelerate_fn_method_page_requests", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_requests_generic_abi: library.func("uniffi_xcelerate_fn_method_page_requests", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_route: library.func("uniffi_xcelerate_fn_method_page_route", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_route_generic_abi: library.func("uniffi_xcelerate_fn_method_page_route", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_route_abort: library.func("uniffi_xcelerate_fn_method_page_route_abort", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_route_abort_generic_abi: library.func("uniffi_xcelerate_fn_method_page_route_abort", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_route_from_har: library.func("uniffi_xcelerate_fn_method_page_route_from_har", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_route_from_har_generic_abi: library.func("uniffi_xcelerate_fn_method_page_route_from_har", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_route_fulfill: library.func("uniffi_xcelerate_fn_method_page_route_fulfill", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi: library.func("uniffi_xcelerate_fn_method_page_route_fulfill", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_screenshot: library.func("uniffi_xcelerate_fn_method_page_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_screenshot_generic_abi: library.func("uniffi_xcelerate_fn_method_page_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_screenshot_full: library.func("uniffi_xcelerate_fn_method_page_screenshot_full", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi: library.func("uniffi_xcelerate_fn_method_page_screenshot_full", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_select_option: library.func("uniffi_xcelerate_fn_method_page_select_option", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_select_option_generic_abi: library.func("uniffi_xcelerate_fn_method_page_select_option", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_cache_enabled: library.func("uniffi_xcelerate_fn_method_page_set_cache_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),

    uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_cache_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),


    uniffi_xcelerate_fn_method_page_set_content: library.func("uniffi_xcelerate_fn_method_page_set_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_content_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_default_timeout: library.func("uniffi_xcelerate_fn_method_page_set_default_timeout", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double"]),

    uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_default_timeout", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double"]),


    uniffi_xcelerate_fn_method_page_set_drag_interception: library.func("uniffi_xcelerate_fn_method_page_set_drag_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),

    uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_drag_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),


    uniffi_xcelerate_fn_method_page_set_emulated_media_features: library.func("uniffi_xcelerate_fn_method_page_set_emulated_media_features", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_emulated_media_features", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_extra_http_headers: library.func("uniffi_xcelerate_fn_method_page_set_extra_http_headers", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_extra_http_headers", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_input_files: library.func("uniffi_xcelerate_fn_method_page_set_input_files", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_input_files_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_input_files", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_javascript_enabled: library.func("uniffi_xcelerate_fn_method_page_set_javascript_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),

    uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_javascript_enabled", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),


    uniffi_xcelerate_fn_method_page_set_offline: library.func("uniffi_xcelerate_fn_method_page_set_offline", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),

    uniffi_xcelerate_fn_method_page_set_offline_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_offline", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),


    uniffi_xcelerate_fn_method_page_set_request_interception: library.func("uniffi_xcelerate_fn_method_page_set_request_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),

    uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_request_interception", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int8_t"]),


    uniffi_xcelerate_fn_method_page_set_storage_state: library.func("uniffi_xcelerate_fn_method_page_set_storage_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_storage_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_user_agent: library.func("uniffi_xcelerate_fn_method_page_set_user_agent", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_user_agent", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_set_viewport_size: library.func("uniffi_xcelerate_fn_method_page_set_viewport_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "uint64_t", "int64_t"]),

    uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_viewport_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "uint64_t", "int64_t"]),


    uniffi_xcelerate_fn_method_page_set_window_bounds: library.func("uniffi_xcelerate_fn_method_page_set_window_bounds", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t", "int64_t", "int64_t"]),

    uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_window_bounds", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t", "int64_t", "int64_t"]),


    uniffi_xcelerate_fn_method_page_set_window_position: library.func("uniffi_xcelerate_fn_method_page_set_window_position", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t"]),

    uniffi_xcelerate_fn_method_page_set_window_position_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_window_position", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t"]),


    uniffi_xcelerate_fn_method_page_set_window_size: library.func("uniffi_xcelerate_fn_method_page_set_window_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t"]),

    uniffi_xcelerate_fn_method_page_set_window_size_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_window_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "int64_t", "int64_t"]),


    uniffi_xcelerate_fn_method_page_set_window_state: library.func("uniffi_xcelerate_fn_method_page_set_window_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_set_window_state_generic_abi: library.func("uniffi_xcelerate_fn_method_page_set_window_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_start_screencast: library.func("uniffi_xcelerate_fn_method_page_start_screencast", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_start_screencast_generic_abi: library.func("uniffi_xcelerate_fn_method_page_start_screencast", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_start_tracing: library.func("uniffi_xcelerate_fn_method_page_start_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_start_tracing_generic_abi: library.func("uniffi_xcelerate_fn_method_page_start_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_stop_screencast: library.func("uniffi_xcelerate_fn_method_page_stop_screencast", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi: library.func("uniffi_xcelerate_fn_method_page_stop_screencast", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_stop_tracing: library.func("uniffi_xcelerate_fn_method_page_stop_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi: library.func("uniffi_xcelerate_fn_method_page_stop_tracing", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_storage_state: library.func("uniffi_xcelerate_fn_method_page_storage_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_storage_state_generic_abi: library.func("uniffi_xcelerate_fn_method_page_storage_state", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_target_id: library.func("uniffi_xcelerate_fn_method_page_target_id", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_page_target_id_generic_abi: library.func("uniffi_xcelerate_fn_method_page_target_id", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_page_title: library.func("uniffi_xcelerate_fn_method_page_title", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_title_generic_abi: library.func("uniffi_xcelerate_fn_method_page_title", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_touch_tap: library.func("uniffi_xcelerate_fn_method_page_touch_tap", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),

    uniffi_xcelerate_fn_method_page_touch_tap_generic_abi: library.func("uniffi_xcelerate_fn_method_page_touch_tap", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, "double", "double"]),


    uniffi_xcelerate_fn_method_page_unroute: library.func("uniffi_xcelerate_fn_method_page_unroute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_unroute_generic_abi: library.func("uniffi_xcelerate_fn_method_page_unroute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_unroute_all: library.func("uniffi_xcelerate_fn_method_page_unroute_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_unroute_all_generic_abi: library.func("uniffi_xcelerate_fn_method_page_unroute_all", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_url: library.func("uniffi_xcelerate_fn_method_page_url", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_url_generic_abi: library.func("uniffi_xcelerate_fn_method_page_url", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_wait_for_event: library.func("uniffi_xcelerate_fn_method_page_wait_for_event", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),

    uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_event", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),


    uniffi_xcelerate_fn_method_page_wait_for_event_default: library.func("uniffi_xcelerate_fn_method_page_wait_for_event_default", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_event_default", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_wait_for_function: library.func("uniffi_xcelerate_fn_method_page_wait_for_function", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),

    uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_function", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),


    uniffi_xcelerate_fn_method_page_wait_for_navigation: library.func("uniffi_xcelerate_fn_method_page_wait_for_navigation", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_navigation", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_wait_for_selector: library.func("uniffi_xcelerate_fn_method_page_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_page_wait_for_xpath: library.func("uniffi_xcelerate_fn_method_page_wait_for_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),

    uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi: library.func("uniffi_xcelerate_fn_method_page_wait_for_xpath", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, "uint64_t"]),


    uniffi_xcelerate_fn_method_page_window_id: library.func("uniffi_xcelerate_fn_method_page_window_id", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_window_id_generic_abi: library.func("uniffi_xcelerate_fn_method_page_window_id", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_window_position: library.func("uniffi_xcelerate_fn_method_page_window_position", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_window_position_generic_abi: library.func("uniffi_xcelerate_fn_method_page_window_position", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_window_rect: library.func("uniffi_xcelerate_fn_method_page_window_rect", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_window_rect_generic_abi: library.func("uniffi_xcelerate_fn_method_page_window_rect", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_method_page_window_size: library.func("uniffi_xcelerate_fn_method_page_window_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_fn_method_page_window_size_generic_abi: library.func("uniffi_xcelerate_fn_method_page_window_size", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_fn_clone_pluginhandle: library.func("uniffi_xcelerate_fn_clone_pluginhandle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_clone_pluginhandle_generic_abi: library.func("uniffi_xcelerate_fn_clone_pluginhandle", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_free_pluginhandle: library.func("uniffi_xcelerate_fn_free_pluginhandle", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_free_pluginhandle_generic_abi: library.func("uniffi_xcelerate_fn_free_pluginhandle", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_pluginhandle_invoke: library.func("uniffi_xcelerate_fn_method_pluginhandle_invoke", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),

    uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi: library.func("uniffi_xcelerate_fn_method_pluginhandle_invoke", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer, ffiTypes.RustBuffer]),


    uniffi_xcelerate_fn_method_pluginhandle_ops: library.func("uniffi_xcelerate_fn_method_pluginhandle_ops", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi: library.func("uniffi_xcelerate_fn_method_pluginhandle_ops", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_fn_method_pluginhandle_plugin_name: library.func("uniffi_xcelerate_fn_method_pluginhandle_plugin_name", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi: library.func("uniffi_xcelerate_fn_method_pluginhandle_plugin_name", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rustbuffer_alloc: library.func("ffi_xcelerate_rustbuffer_alloc", ffiTypes.RustBuffer, ["uint64_t", koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rustbuffer_from_bytes: library.func("ffi_xcelerate_rustbuffer_from_bytes", ffiTypes.RustBuffer, [ffiTypes.ForeignBytes, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rustbuffer_free: library.func("ffi_xcelerate_rustbuffer_free", "void", [ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rustbuffer_reserve: library.func("ffi_xcelerate_rustbuffer_reserve", ffiTypes.RustBuffer, [ffiTypes.RustBuffer, "uint64_t", koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_u8: library.func("ffi_xcelerate_rust_future_poll_u8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_u8_generic_abi: library.func("ffi_xcelerate_rust_future_poll_u8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_u8: library.func("ffi_xcelerate_rust_future_cancel_u8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_u8_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_u8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_u8: library.func("ffi_xcelerate_rust_future_free_u8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_u8_generic_abi: library.func("ffi_xcelerate_rust_future_free_u8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_u8: library.func("ffi_xcelerate_rust_future_complete_u8", "uint8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_u8_generic_abi: library.func("ffi_xcelerate_rust_future_complete_u8", "uint8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_i8: library.func("ffi_xcelerate_rust_future_poll_i8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_i8_generic_abi: library.func("ffi_xcelerate_rust_future_poll_i8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_i8: library.func("ffi_xcelerate_rust_future_cancel_i8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_i8_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_i8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_i8: library.func("ffi_xcelerate_rust_future_free_i8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_i8_generic_abi: library.func("ffi_xcelerate_rust_future_free_i8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_i8: library.func("ffi_xcelerate_rust_future_complete_i8", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_i8_generic_abi: library.func("ffi_xcelerate_rust_future_complete_i8", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_u16: library.func("ffi_xcelerate_rust_future_poll_u16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_u16_generic_abi: library.func("ffi_xcelerate_rust_future_poll_u16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_u16: library.func("ffi_xcelerate_rust_future_cancel_u16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_u16_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_u16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_u16: library.func("ffi_xcelerate_rust_future_free_u16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_u16_generic_abi: library.func("ffi_xcelerate_rust_future_free_u16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_u16: library.func("ffi_xcelerate_rust_future_complete_u16", "uint16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_u16_generic_abi: library.func("ffi_xcelerate_rust_future_complete_u16", "uint16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_i16: library.func("ffi_xcelerate_rust_future_poll_i16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_i16_generic_abi: library.func("ffi_xcelerate_rust_future_poll_i16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_i16: library.func("ffi_xcelerate_rust_future_cancel_i16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_i16_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_i16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_i16: library.func("ffi_xcelerate_rust_future_free_i16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_i16_generic_abi: library.func("ffi_xcelerate_rust_future_free_i16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_i16: library.func("ffi_xcelerate_rust_future_complete_i16", "int16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_i16_generic_abi: library.func("ffi_xcelerate_rust_future_complete_i16", "int16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_u32: library.func("ffi_xcelerate_rust_future_poll_u32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_u32_generic_abi: library.func("ffi_xcelerate_rust_future_poll_u32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_u32: library.func("ffi_xcelerate_rust_future_cancel_u32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_u32_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_u32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_u32: library.func("ffi_xcelerate_rust_future_free_u32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_u32_generic_abi: library.func("ffi_xcelerate_rust_future_free_u32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_u32: library.func("ffi_xcelerate_rust_future_complete_u32", "uint32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_u32_generic_abi: library.func("ffi_xcelerate_rust_future_complete_u32", "uint32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_i32: library.func("ffi_xcelerate_rust_future_poll_i32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_i32_generic_abi: library.func("ffi_xcelerate_rust_future_poll_i32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_i32: library.func("ffi_xcelerate_rust_future_cancel_i32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_i32_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_i32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_i32: library.func("ffi_xcelerate_rust_future_free_i32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_i32_generic_abi: library.func("ffi_xcelerate_rust_future_free_i32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_i32: library.func("ffi_xcelerate_rust_future_complete_i32", "int32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_i32_generic_abi: library.func("ffi_xcelerate_rust_future_complete_i32", "int32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_u64: library.func("ffi_xcelerate_rust_future_poll_u64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_u64_generic_abi: library.func("ffi_xcelerate_rust_future_poll_u64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_u64: library.func("ffi_xcelerate_rust_future_cancel_u64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_u64_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_u64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_u64: library.func("ffi_xcelerate_rust_future_free_u64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_u64_generic_abi: library.func("ffi_xcelerate_rust_future_free_u64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_u64: library.func("ffi_xcelerate_rust_future_complete_u64", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_u64_generic_abi: library.func("ffi_xcelerate_rust_future_complete_u64", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_i64: library.func("ffi_xcelerate_rust_future_poll_i64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_i64_generic_abi: library.func("ffi_xcelerate_rust_future_poll_i64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_i64: library.func("ffi_xcelerate_rust_future_cancel_i64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_i64_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_i64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_i64: library.func("ffi_xcelerate_rust_future_free_i64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_i64_generic_abi: library.func("ffi_xcelerate_rust_future_free_i64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_i64: library.func("ffi_xcelerate_rust_future_complete_i64", "int64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_i64_generic_abi: library.func("ffi_xcelerate_rust_future_complete_i64", "int64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_f32: library.func("ffi_xcelerate_rust_future_poll_f32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_f32_generic_abi: library.func("ffi_xcelerate_rust_future_poll_f32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_f32: library.func("ffi_xcelerate_rust_future_cancel_f32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_f32_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_f32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_f32: library.func("ffi_xcelerate_rust_future_free_f32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_f32_generic_abi: library.func("ffi_xcelerate_rust_future_free_f32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_f32: library.func("ffi_xcelerate_rust_future_complete_f32", "float", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_f32_generic_abi: library.func("ffi_xcelerate_rust_future_complete_f32", "float", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_f64: library.func("ffi_xcelerate_rust_future_poll_f64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_f64_generic_abi: library.func("ffi_xcelerate_rust_future_poll_f64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_f64: library.func("ffi_xcelerate_rust_future_cancel_f64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_f64_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_f64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_f64: library.func("ffi_xcelerate_rust_future_free_f64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_f64_generic_abi: library.func("ffi_xcelerate_rust_future_free_f64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_f64: library.func("ffi_xcelerate_rust_future_complete_f64", "double", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_f64_generic_abi: library.func("ffi_xcelerate_rust_future_complete_f64", "double", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_rust_buffer: library.func("ffi_xcelerate_rust_future_poll_rust_buffer", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_rust_buffer_generic_abi: library.func("ffi_xcelerate_rust_future_poll_rust_buffer", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_rust_buffer: library.func("ffi_xcelerate_rust_future_cancel_rust_buffer", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_rust_buffer_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_rust_buffer", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_rust_buffer: library.func("ffi_xcelerate_rust_future_free_rust_buffer", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_rust_buffer_generic_abi: library.func("ffi_xcelerate_rust_future_free_rust_buffer", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_rust_buffer: library.func("ffi_xcelerate_rust_future_complete_rust_buffer", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_rust_buffer_generic_abi: library.func("ffi_xcelerate_rust_future_complete_rust_buffer", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_rust_future_poll_void: library.func("ffi_xcelerate_rust_future_poll_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_poll_void_generic_abi: library.func("ffi_xcelerate_rust_future_poll_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_cancel_void: library.func("ffi_xcelerate_rust_future_cancel_void", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_cancel_void_generic_abi: library.func("ffi_xcelerate_rust_future_cancel_void", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_free_void: library.func("ffi_xcelerate_rust_future_free_void", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_rust_future_free_void_generic_abi: library.func("ffi_xcelerate_rust_future_free_void", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_rust_future_complete_void: library.func("ffi_xcelerate_rust_future_complete_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_rust_future_complete_void_generic_abi: library.func("ffi_xcelerate_rust_future_complete_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_checksum_method_browser_audit_log: library.func("uniffi_xcelerate_checksum_method_browser_audit_log", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_audit_verify: library.func("uniffi_xcelerate_checksum_method_browser_audit_verify", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_available_plugins: library.func("uniffi_xcelerate_checksum_method_browser_available_plugins", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_browser_contexts: library.func("uniffi_xcelerate_checksum_method_browser_browser_contexts", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_capabilities: library.func("uniffi_xcelerate_checksum_method_browser_capabilities", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_close: library.func("uniffi_xcelerate_checksum_method_browser_close", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_cookies: library.func("uniffi_xcelerate_checksum_method_browser_cookies", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_delete_cookie: library.func("uniffi_xcelerate_checksum_method_browser_delete_cookie", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_event_names: library.func("uniffi_xcelerate_checksum_method_browser_event_names", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_grant_permissions: library.func("uniffi_xcelerate_checksum_method_browser_grant_permissions", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_is_connected: library.func("uniffi_xcelerate_checksum_method_browser_is_connected", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_listens_to: library.func("uniffi_xcelerate_checksum_method_browser_listens_to", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_load_plugin: library.func("uniffi_xcelerate_checksum_method_browser_load_plugin", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_new_context: library.func("uniffi_xcelerate_checksum_method_browser_new_context", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_new_page: library.func("uniffi_xcelerate_checksum_method_browser_new_page", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_on: library.func("uniffi_xcelerate_checksum_method_browser_on", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_once: library.func("uniffi_xcelerate_checksum_method_browser_once", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_plugin: library.func("uniffi_xcelerate_checksum_method_browser_plugin", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_plugin_names: library.func("uniffi_xcelerate_checksum_method_browser_plugin_names", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_remove_all_listeners: library.func("uniffi_xcelerate_checksum_method_browser_remove_all_listeners", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_remove_listener: library.func("uniffi_xcelerate_checksum_method_browser_remove_listener", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_reset_permissions: library.func("uniffi_xcelerate_checksum_method_browser_reset_permissions", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_set_cookie: library.func("uniffi_xcelerate_checksum_method_browser_set_cookie", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_set_download_behavior: library.func("uniffi_xcelerate_checksum_method_browser_set_download_behavior", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_start_tracing: library.func("uniffi_xcelerate_checksum_method_browser_start_tracing", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_stop_tracing: library.func("uniffi_xcelerate_checksum_method_browser_stop_tracing", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_targets: library.func("uniffi_xcelerate_checksum_method_browser_targets", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_use_plugin: library.func("uniffi_xcelerate_checksum_method_browser_use_plugin", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_user_agent: library.func("uniffi_xcelerate_checksum_method_browser_user_agent", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_version: library.func("uniffi_xcelerate_checksum_method_browser_version", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_wait_for_event: library.func("uniffi_xcelerate_checksum_method_browser_wait_for_event", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_wait_for_event_default: library.func("uniffi_xcelerate_checksum_method_browser_wait_for_event_default", "uint16_t", []),


    uniffi_xcelerate_checksum_method_browser_ws_endpoint: library.func("uniffi_xcelerate_checksum_method_browser_ws_endpoint", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_attribute: library.func("uniffi_xcelerate_checksum_method_element_attribute", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_call_bool: library.func("uniffi_xcelerate_checksum_method_element_call_bool", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_call_json: library.func("uniffi_xcelerate_checksum_method_element_call_json", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_call_on_selector: library.func("uniffi_xcelerate_checksum_method_element_call_on_selector", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_call_on_selector_all: library.func("uniffi_xcelerate_checksum_method_element_call_on_selector_all", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_call_string: library.func("uniffi_xcelerate_checksum_method_element_call_string", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_click: library.func("uniffi_xcelerate_checksum_method_element_click", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_click_mouse: library.func("uniffi_xcelerate_checksum_method_element_click_mouse", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_count: library.func("uniffi_xcelerate_checksum_method_element_count", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_dispose: library.func("uniffi_xcelerate_checksum_method_element_dispose", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_evaluate_bool: library.func("uniffi_xcelerate_checksum_method_element_evaluate_bool", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_evaluate_handle: library.func("uniffi_xcelerate_checksum_method_element_evaluate_handle", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_evaluate_json: library.func("uniffi_xcelerate_checksum_method_element_evaluate_json", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_evaluate_string: library.func("uniffi_xcelerate_checksum_method_element_evaluate_string", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_focus: library.func("uniffi_xcelerate_checksum_method_element_focus", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_get_by_label: library.func("uniffi_xcelerate_checksum_method_element_get_by_label", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_get_by_role: library.func("uniffi_xcelerate_checksum_method_element_get_by_role", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_get_by_text: library.func("uniffi_xcelerate_checksum_method_element_get_by_text", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_get_properties: library.func("uniffi_xcelerate_checksum_method_element_get_properties", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_hover: library.func("uniffi_xcelerate_checksum_method_element_hover", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_hover_mouse: library.func("uniffi_xcelerate_checksum_method_element_hover_mouse", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_inner_html: library.func("uniffi_xcelerate_checksum_method_element_inner_html", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_press: library.func("uniffi_xcelerate_checksum_method_element_press", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_query_selector: library.func("uniffi_xcelerate_checksum_method_element_query_selector", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_query_selector_all: library.func("uniffi_xcelerate_checksum_method_element_query_selector_all", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_query_selector_attr: library.func("uniffi_xcelerate_checksum_method_element_query_selector_attr", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_query_selector_xpath: library.func("uniffi_xcelerate_checksum_method_element_query_selector_xpath", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_screenshot: library.func("uniffi_xcelerate_checksum_method_element_screenshot", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_screenshot_base64: library.func("uniffi_xcelerate_checksum_method_element_screenshot_base64", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_select_option: library.func("uniffi_xcelerate_checksum_method_element_select_option", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_set_input_files: library.func("uniffi_xcelerate_checksum_method_element_set_input_files", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_text: library.func("uniffi_xcelerate_checksum_method_element_text", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_type_text: library.func("uniffi_xcelerate_checksum_method_element_type_text", "uint16_t", []),


    uniffi_xcelerate_checksum_method_element_wait_for_selector: library.func("uniffi_xcelerate_checksum_method_element_wait_for_selector", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_activate: library.func("uniffi_xcelerate_checksum_method_page_activate", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_activate_target: library.func("uniffi_xcelerate_checksum_method_page_activate_target", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document: library.func("uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_add_style_tag: library.func("uniffi_xcelerate_checksum_method_page_add_style_tag", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_authenticate: library.func("uniffi_xcelerate_checksum_method_page_authenticate", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_bring_to_front: library.func("uniffi_xcelerate_checksum_method_page_bring_to_front", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_call_bool: library.func("uniffi_xcelerate_checksum_method_page_call_bool", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_call_json: library.func("uniffi_xcelerate_checksum_method_page_call_json", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_call_on_selector: library.func("uniffi_xcelerate_checksum_method_page_call_on_selector", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_call_on_selector_all: library.func("uniffi_xcelerate_checksum_method_page_call_on_selector_all", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_call_string: library.func("uniffi_xcelerate_checksum_method_page_call_string", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_clear_requests: library.func("uniffi_xcelerate_checksum_method_page_clear_requests", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_click_mouse: library.func("uniffi_xcelerate_checksum_method_page_click_mouse", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_close: library.func("uniffi_xcelerate_checksum_method_page_close", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_content: library.func("uniffi_xcelerate_checksum_method_page_content", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_cookie: library.func("uniffi_xcelerate_checksum_method_page_cookie", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_cookies: library.func("uniffi_xcelerate_checksum_method_page_cookies", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_coverage_start_css: library.func("uniffi_xcelerate_checksum_method_page_coverage_start_css", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_coverage_start_js: library.func("uniffi_xcelerate_checksum_method_page_coverage_start_js", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_coverage_stop_css: library.func("uniffi_xcelerate_checksum_method_page_coverage_stop_css", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_coverage_stop_js: library.func("uniffi_xcelerate_checksum_method_page_coverage_stop_js", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_create_pdf_stream: library.func("uniffi_xcelerate_checksum_method_page_create_pdf_stream", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_decode_base64: library.func("uniffi_xcelerate_checksum_method_page_decode_base64", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_default_timeout: library.func("uniffi_xcelerate_checksum_method_page_default_timeout", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_document_element: library.func("uniffi_xcelerate_checksum_method_page_document_element", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_emulate_idle_state: library.func("uniffi_xcelerate_checksum_method_page_emulate_idle_state", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_emulate_media: library.func("uniffi_xcelerate_checksum_method_page_emulate_media", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_ensure_interception: library.func("uniffi_xcelerate_checksum_method_page_ensure_interception", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_evaluate_bool: library.func("uniffi_xcelerate_checksum_method_page_evaluate_bool", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_evaluate_handle: library.func("uniffi_xcelerate_checksum_method_page_evaluate_handle", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_evaluate_json: library.func("uniffi_xcelerate_checksum_method_page_evaluate_json", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_evaluate_string: library.func("uniffi_xcelerate_checksum_method_page_evaluate_string", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_event_names: library.func("uniffi_xcelerate_checksum_method_page_event_names", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_execute_cdp_cmd: library.func("uniffi_xcelerate_checksum_method_page_execute_cdp_cmd", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_find_element: library.func("uniffi_xcelerate_checksum_method_page_find_element", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_frame: library.func("uniffi_xcelerate_checksum_method_page_frame", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_frame_name: library.func("uniffi_xcelerate_checksum_method_page_frame_name", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_frames: library.func("uniffi_xcelerate_checksum_method_page_frames", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_get_by_label: library.func("uniffi_xcelerate_checksum_method_page_get_by_label", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_get_by_role: library.func("uniffi_xcelerate_checksum_method_page_get_by_role", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_get_by_text: library.func("uniffi_xcelerate_checksum_method_page_get_by_text", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_get_default_timeout: library.func("uniffi_xcelerate_checksum_method_page_get_default_timeout", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_go_back: library.func("uniffi_xcelerate_checksum_method_page_go_back", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_go_forward: library.func("uniffi_xcelerate_checksum_method_page_go_forward", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_handle_js_dialog: library.func("uniffi_xcelerate_checksum_method_page_handle_js_dialog", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_inject_file: library.func("uniffi_xcelerate_checksum_method_page_inject_file", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled: library.func("uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_keyboard_down: library.func("uniffi_xcelerate_checksum_method_page_keyboard_down", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_keyboard_press: library.func("uniffi_xcelerate_checksum_method_page_keyboard_press", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_keyboard_type: library.func("uniffi_xcelerate_checksum_method_page_keyboard_type", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_keyboard_up: library.func("uniffi_xcelerate_checksum_method_page_keyboard_up", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_listens_to: library.func("uniffi_xcelerate_checksum_method_page_listens_to", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_main_frame: library.func("uniffi_xcelerate_checksum_method_page_main_frame", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_metrics: library.func("uniffi_xcelerate_checksum_method_page_metrics", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_mouse_down: library.func("uniffi_xcelerate_checksum_method_page_mouse_down", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_mouse_up: library.func("uniffi_xcelerate_checksum_method_page_mouse_up", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_move_mouse: library.func("uniffi_xcelerate_checksum_method_page_move_mouse", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_navigate: library.func("uniffi_xcelerate_checksum_method_page_navigate", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_on: library.func("uniffi_xcelerate_checksum_method_page_on", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_once: library.func("uniffi_xcelerate_checksum_method_page_once", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_pdf: library.func("uniffi_xcelerate_checksum_method_page_pdf", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_press: library.func("uniffi_xcelerate_checksum_method_page_press", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_query_selector_all: library.func("uniffi_xcelerate_checksum_method_page_query_selector_all", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_query_selector_xpath: library.func("uniffi_xcelerate_checksum_method_page_query_selector_xpath", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_raw_window_bounds: library.func("uniffi_xcelerate_checksum_method_page_raw_window_bounds", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_reload: library.func("uniffi_xcelerate_checksum_method_page_reload", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_remove_all_listeners: library.func("uniffi_xcelerate_checksum_method_page_remove_all_listeners", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_remove_listener: library.func("uniffi_xcelerate_checksum_method_page_remove_listener", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_remove_script: library.func("uniffi_xcelerate_checksum_method_page_remove_script", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_request: library.func("uniffi_xcelerate_checksum_method_page_request", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_requests: library.func("uniffi_xcelerate_checksum_method_page_requests", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_route: library.func("uniffi_xcelerate_checksum_method_page_route", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_route_abort: library.func("uniffi_xcelerate_checksum_method_page_route_abort", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_route_from_har: library.func("uniffi_xcelerate_checksum_method_page_route_from_har", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_route_fulfill: library.func("uniffi_xcelerate_checksum_method_page_route_fulfill", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_screenshot: library.func("uniffi_xcelerate_checksum_method_page_screenshot", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_screenshot_full: library.func("uniffi_xcelerate_checksum_method_page_screenshot_full", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_select_option: library.func("uniffi_xcelerate_checksum_method_page_select_option", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_cache_enabled: library.func("uniffi_xcelerate_checksum_method_page_set_cache_enabled", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_content: library.func("uniffi_xcelerate_checksum_method_page_set_content", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_default_timeout: library.func("uniffi_xcelerate_checksum_method_page_set_default_timeout", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_drag_interception: library.func("uniffi_xcelerate_checksum_method_page_set_drag_interception", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_emulated_media_features: library.func("uniffi_xcelerate_checksum_method_page_set_emulated_media_features", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_extra_http_headers: library.func("uniffi_xcelerate_checksum_method_page_set_extra_http_headers", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_input_files: library.func("uniffi_xcelerate_checksum_method_page_set_input_files", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_javascript_enabled: library.func("uniffi_xcelerate_checksum_method_page_set_javascript_enabled", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_offline: library.func("uniffi_xcelerate_checksum_method_page_set_offline", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_request_interception: library.func("uniffi_xcelerate_checksum_method_page_set_request_interception", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_storage_state: library.func("uniffi_xcelerate_checksum_method_page_set_storage_state", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_user_agent: library.func("uniffi_xcelerate_checksum_method_page_set_user_agent", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_viewport_size: library.func("uniffi_xcelerate_checksum_method_page_set_viewport_size", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_window_bounds: library.func("uniffi_xcelerate_checksum_method_page_set_window_bounds", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_window_position: library.func("uniffi_xcelerate_checksum_method_page_set_window_position", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_window_size: library.func("uniffi_xcelerate_checksum_method_page_set_window_size", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_set_window_state: library.func("uniffi_xcelerate_checksum_method_page_set_window_state", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_start_screencast: library.func("uniffi_xcelerate_checksum_method_page_start_screencast", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_start_tracing: library.func("uniffi_xcelerate_checksum_method_page_start_tracing", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_stop_screencast: library.func("uniffi_xcelerate_checksum_method_page_stop_screencast", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_stop_tracing: library.func("uniffi_xcelerate_checksum_method_page_stop_tracing", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_storage_state: library.func("uniffi_xcelerate_checksum_method_page_storage_state", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_target_id: library.func("uniffi_xcelerate_checksum_method_page_target_id", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_title: library.func("uniffi_xcelerate_checksum_method_page_title", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_touch_tap: library.func("uniffi_xcelerate_checksum_method_page_touch_tap", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_unroute: library.func("uniffi_xcelerate_checksum_method_page_unroute", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_unroute_all: library.func("uniffi_xcelerate_checksum_method_page_unroute_all", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_url: library.func("uniffi_xcelerate_checksum_method_page_url", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_event: library.func("uniffi_xcelerate_checksum_method_page_wait_for_event", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_event_default: library.func("uniffi_xcelerate_checksum_method_page_wait_for_event_default", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_function: library.func("uniffi_xcelerate_checksum_method_page_wait_for_function", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_navigation: library.func("uniffi_xcelerate_checksum_method_page_wait_for_navigation", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_selector: library.func("uniffi_xcelerate_checksum_method_page_wait_for_selector", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_wait_for_xpath: library.func("uniffi_xcelerate_checksum_method_page_wait_for_xpath", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_window_id: library.func("uniffi_xcelerate_checksum_method_page_window_id", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_window_position: library.func("uniffi_xcelerate_checksum_method_page_window_position", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_window_rect: library.func("uniffi_xcelerate_checksum_method_page_window_rect", "uint16_t", []),


    uniffi_xcelerate_checksum_method_page_window_size: library.func("uniffi_xcelerate_checksum_method_page_window_size", "uint16_t", []),


    uniffi_xcelerate_checksum_method_pluginhandle_invoke: library.func("uniffi_xcelerate_checksum_method_pluginhandle_invoke", "uint16_t", []),


    uniffi_xcelerate_checksum_method_pluginhandle_ops: library.func("uniffi_xcelerate_checksum_method_pluginhandle_ops", "uint16_t", []),


    uniffi_xcelerate_checksum_method_pluginhandle_plugin_name: library.func("uniffi_xcelerate_checksum_method_pluginhandle_plugin_name", "uint16_t", []),


    uniffi_xcelerate_checksum_constructor_browser_launch: library.func("uniffi_xcelerate_checksum_constructor_browser_launch", "uint16_t", []),


    ffi_xcelerate_uniffi_contract_version: library.func("ffi_xcelerate_uniffi_contract_version", "uint32_t", []),


  });

  return Object.freeze({
    library,
    ffiTypes,
    ffiCallbacks,
    ffiStructs,
    ffiFunctions,
  });
}

function createBindings(libraryPath, bindingCore = undefined, resolution = undefined) {
  const core = bindingCore ?? createBindingCore(libraryPath);
  const packageRelativePath = resolution?.packageRelativePath ?? null;
  return Object.freeze({
    libraryPath,
    packageRelativePath,
    library: core.library,
    ffiTypes: core.ffiTypes,
    ffiCallbacks: core.ffiCallbacks,
    ffiStructs: core.ffiStructs,
    ffiFunctions: core.ffiFunctions,
  });
}

function cacheBindingCore(libraryPath, bindings) {
  cachedLibraryPath = libraryPath;
  cachedBindingCore = Object.freeze({
    library: bindings.library,
    ffiTypes: bindings.ffiTypes,
    ffiCallbacks: bindings.ffiCallbacks,
    ffiStructs: bindings.ffiStructs,
    ffiFunctions: bindings.ffiFunctions,
  });
  return cachedBindingCore;
}

function clearBindingCoreCache() {
  cachedBindingCore = null;
  cachedLibraryPath = null;
}

export function load(libraryPath = undefined) {
  const resolution = resolveLibraryPath(libraryPath);
  const resolvedLibraryPath = resolution.libraryPath;
  const packageRelativePath = resolution.packageRelativePath;
  const bundledPrebuild = resolution.bundledPrebuild;
  const canonicalLibraryPath = canonicalizeExistingLibraryPath(resolvedLibraryPath);

  if (loadedBindings !== null) {
    if (loadedBindings.libraryPath === canonicalLibraryPath) {
      return loadedBindings;
    }

    throw new Error(
      `The native library is already loaded from ${JSON.stringify(loadedBindings.libraryPath)}. Call unload() before loading a different library path.`,
    );
  }

  if (packageRelativePath !== null && !existsSync(resolvedLibraryPath)) {
    if (bundledPrebuild !== null) {
      throw new Error(
        `No bundled UniFFI library was found for target ${JSON.stringify(bundledPrebuild.target)}. The generated package expects ${JSON.stringify(bundledPrebuild.packageRelativePath)} at ${JSON.stringify(resolvedLibraryPath)}.`,
      );
    }

    throw new Error(
      `No packaged UniFFI library was found at ${JSON.stringify(packageRelativePath)}. The generated package expects ${JSON.stringify(resolvedLibraryPath)}.`,
    );
  }

  let bindingCore =
    cachedLibraryPath === canonicalLibraryPath
      ? cachedBindingCore
      : null;
  if (bindingCore == null && cachedBindingCore != null) {
    cachedBindingCore.library.unload();
    clearBindingCoreCache();
  }

  const bindings = createBindings(canonicalLibraryPath, bindingCore, resolution);
  try {
    runtimeHooks.onLoad?.(bindings);
    if (bindingCore == null) {
      validateContractVersion(bindings);
      validateChecksums(bindings);
      bindingCore = cacheBindingCore(canonicalLibraryPath, bindings);
    }
  } catch (error) {
    try {
      runtimeHooks.onUnload?.(bindings);
    } catch {
      // Preserve the original initialization failure.
    }
    if (bindingCore == null) {
      try {
        bindings.library.unload();
      } catch {
        // Preserve the original initialization failure.
      }
    }
    throw error;
  }

  loadedBindings = bindings;
  loadedFfiTypes = bindings.ffiTypes;
  loadedFfiFunctions = bindings.ffiFunctions;
  return loadedBindings;
}

export function unload() {
  if (loadedBindings === null) {
    return false;
  }

  let hookError = null;
  try {
    runtimeHooks.onUnload?.(loadedBindings);
  } catch (error) {
    hookError = error;
  }
  loadedBindings = null;
  loadedFfiTypes = null;
  loadedFfiFunctions = null;
  if (hookError != null) {
    throw hookError;
  }
  return true;
}

export function isLoaded() {
  return loadedBindings !== null;
}

export function configureRuntimeHooks(hooks = undefined) {
  runtimeHooks = Object.freeze(hooks ?? {});
}

function throwLibraryNotLoaded() {
  throw new LibraryNotLoadedError(libraryNotLoadedMessage);
}

export function getFfiBindings() {
  if (loadedBindings === null) {
    throwLibraryNotLoaded();
  }

  return loadedBindings;
}

export function getFfiTypes() {
  if (loadedFfiTypes === null) {
    throwLibraryNotLoaded();
  }

  return loadedFfiTypes;
}

function getLoadedFfiFunctions() {
  if (loadedFfiFunctions === null) {
    throwLibraryNotLoaded();
  }

  return loadedFfiFunctions;
}

export function getContractVersion(bindings = getFfiBindings()) {
  return bindings.ffiFunctions.ffi_xcelerate_uniffi_contract_version();
}

export function validateContractVersion(bindings = getFfiBindings()) {
  const actual = getContractVersion(bindings);
  const expected = ffiIntegrity.expectedContractVersion;
  if (actual !== expected) {
    throw new ContractVersionMismatchError(expected, actual, {
      details: {
        libraryPath: bindings.libraryPath,
        packageRelativePath: bindings.packageRelativePath,
        symbolName: ffiIntegrity.contractVersionFunction,
      },
    });
  }
  return actual;
}

export function getChecksums(bindings = getFfiBindings()) {
  return Object.freeze({

    "uniffi_xcelerate_checksum_method_browser_audit_log": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_audit_log(),

    "uniffi_xcelerate_checksum_method_browser_audit_verify": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_audit_verify(),

    "uniffi_xcelerate_checksum_method_browser_available_plugins": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_available_plugins(),

    "uniffi_xcelerate_checksum_method_browser_browser_contexts": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_browser_contexts(),

    "uniffi_xcelerate_checksum_method_browser_capabilities": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_capabilities(),

    "uniffi_xcelerate_checksum_method_browser_close": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_close(),

    "uniffi_xcelerate_checksum_method_browser_cookies": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_cookies(),

    "uniffi_xcelerate_checksum_method_browser_delete_cookie": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_delete_cookie(),

    "uniffi_xcelerate_checksum_method_browser_event_names": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_event_names(),

    "uniffi_xcelerate_checksum_method_browser_grant_permissions": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_grant_permissions(),

    "uniffi_xcelerate_checksum_method_browser_is_connected": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_is_connected(),

    "uniffi_xcelerate_checksum_method_browser_listens_to": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_listens_to(),

    "uniffi_xcelerate_checksum_method_browser_load_plugin": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_load_plugin(),

    "uniffi_xcelerate_checksum_method_browser_new_context": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_new_context(),

    "uniffi_xcelerate_checksum_method_browser_new_page": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_new_page(),

    "uniffi_xcelerate_checksum_method_browser_on": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_on(),

    "uniffi_xcelerate_checksum_method_browser_once": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_once(),

    "uniffi_xcelerate_checksum_method_browser_plugin": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_plugin(),

    "uniffi_xcelerate_checksum_method_browser_plugin_names": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_plugin_names(),

    "uniffi_xcelerate_checksum_method_browser_remove_all_listeners": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_remove_all_listeners(),

    "uniffi_xcelerate_checksum_method_browser_remove_listener": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_remove_listener(),

    "uniffi_xcelerate_checksum_method_browser_reset_permissions": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_reset_permissions(),

    "uniffi_xcelerate_checksum_method_browser_set_cookie": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_set_cookie(),

    "uniffi_xcelerate_checksum_method_browser_set_download_behavior": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_set_download_behavior(),

    "uniffi_xcelerate_checksum_method_browser_start_tracing": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_start_tracing(),

    "uniffi_xcelerate_checksum_method_browser_stop_tracing": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_stop_tracing(),

    "uniffi_xcelerate_checksum_method_browser_targets": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_targets(),

    "uniffi_xcelerate_checksum_method_browser_use_plugin": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_use_plugin(),

    "uniffi_xcelerate_checksum_method_browser_user_agent": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_user_agent(),

    "uniffi_xcelerate_checksum_method_browser_version": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_version(),

    "uniffi_xcelerate_checksum_method_browser_wait_for_event": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_wait_for_event(),

    "uniffi_xcelerate_checksum_method_browser_wait_for_event_default": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_wait_for_event_default(),

    "uniffi_xcelerate_checksum_method_browser_ws_endpoint": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_browser_ws_endpoint(),

    "uniffi_xcelerate_checksum_method_element_attribute": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_attribute(),

    "uniffi_xcelerate_checksum_method_element_call_bool": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_call_bool(),

    "uniffi_xcelerate_checksum_method_element_call_json": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_call_json(),

    "uniffi_xcelerate_checksum_method_element_call_on_selector": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_call_on_selector(),

    "uniffi_xcelerate_checksum_method_element_call_on_selector_all": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_call_on_selector_all(),

    "uniffi_xcelerate_checksum_method_element_call_string": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_call_string(),

    "uniffi_xcelerate_checksum_method_element_click": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_click(),

    "uniffi_xcelerate_checksum_method_element_click_mouse": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_click_mouse(),

    "uniffi_xcelerate_checksum_method_element_count": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_count(),

    "uniffi_xcelerate_checksum_method_element_dispose": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_dispose(),

    "uniffi_xcelerate_checksum_method_element_evaluate_bool": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_bool(),

    "uniffi_xcelerate_checksum_method_element_evaluate_handle": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_handle(),

    "uniffi_xcelerate_checksum_method_element_evaluate_json": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_json(),

    "uniffi_xcelerate_checksum_method_element_evaluate_string": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_string(),

    "uniffi_xcelerate_checksum_method_element_focus": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_focus(),

    "uniffi_xcelerate_checksum_method_element_get_by_label": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_label(),

    "uniffi_xcelerate_checksum_method_element_get_by_role": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_role(),

    "uniffi_xcelerate_checksum_method_element_get_by_text": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_text(),

    "uniffi_xcelerate_checksum_method_element_get_properties": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_get_properties(),

    "uniffi_xcelerate_checksum_method_element_hover": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_hover(),

    "uniffi_xcelerate_checksum_method_element_hover_mouse": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_hover_mouse(),

    "uniffi_xcelerate_checksum_method_element_inner_html": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_inner_html(),

    "uniffi_xcelerate_checksum_method_element_press": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_press(),

    "uniffi_xcelerate_checksum_method_element_query_selector": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector(),

    "uniffi_xcelerate_checksum_method_element_query_selector_all": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_all(),

    "uniffi_xcelerate_checksum_method_element_query_selector_attr": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_attr(),

    "uniffi_xcelerate_checksum_method_element_query_selector_xpath": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_xpath(),

    "uniffi_xcelerate_checksum_method_element_screenshot": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_screenshot(),

    "uniffi_xcelerate_checksum_method_element_screenshot_base64": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_screenshot_base64(),

    "uniffi_xcelerate_checksum_method_element_select_option": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_select_option(),

    "uniffi_xcelerate_checksum_method_element_set_input_files": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_set_input_files(),

    "uniffi_xcelerate_checksum_method_element_text": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_text(),

    "uniffi_xcelerate_checksum_method_element_type_text": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_type_text(),

    "uniffi_xcelerate_checksum_method_element_wait_for_selector": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_element_wait_for_selector(),

    "uniffi_xcelerate_checksum_method_page_activate": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_activate(),

    "uniffi_xcelerate_checksum_method_page_activate_target": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_activate_target(),

    "uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(),

    "uniffi_xcelerate_checksum_method_page_add_style_tag": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_add_style_tag(),

    "uniffi_xcelerate_checksum_method_page_authenticate": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_authenticate(),

    "uniffi_xcelerate_checksum_method_page_bring_to_front": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_bring_to_front(),

    "uniffi_xcelerate_checksum_method_page_call_bool": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_call_bool(),

    "uniffi_xcelerate_checksum_method_page_call_json": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_call_json(),

    "uniffi_xcelerate_checksum_method_page_call_on_selector": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_call_on_selector(),

    "uniffi_xcelerate_checksum_method_page_call_on_selector_all": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_call_on_selector_all(),

    "uniffi_xcelerate_checksum_method_page_call_string": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_call_string(),

    "uniffi_xcelerate_checksum_method_page_clear_requests": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_clear_requests(),

    "uniffi_xcelerate_checksum_method_page_click_mouse": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_click_mouse(),

    "uniffi_xcelerate_checksum_method_page_close": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_close(),

    "uniffi_xcelerate_checksum_method_page_content": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_content(),

    "uniffi_xcelerate_checksum_method_page_cookie": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_cookie(),

    "uniffi_xcelerate_checksum_method_page_cookies": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_cookies(),

    "uniffi_xcelerate_checksum_method_page_coverage_start_css": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_start_css(),

    "uniffi_xcelerate_checksum_method_page_coverage_start_js": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_start_js(),

    "uniffi_xcelerate_checksum_method_page_coverage_stop_css": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_stop_css(),

    "uniffi_xcelerate_checksum_method_page_coverage_stop_js": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_stop_js(),

    "uniffi_xcelerate_checksum_method_page_create_pdf_stream": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_create_pdf_stream(),

    "uniffi_xcelerate_checksum_method_page_decode_base64": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_decode_base64(),

    "uniffi_xcelerate_checksum_method_page_default_timeout": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_default_timeout(),

    "uniffi_xcelerate_checksum_method_page_document_element": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_document_element(),

    "uniffi_xcelerate_checksum_method_page_emulate_idle_state": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_emulate_idle_state(),

    "uniffi_xcelerate_checksum_method_page_emulate_media": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_emulate_media(),

    "uniffi_xcelerate_checksum_method_page_ensure_interception": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_ensure_interception(),

    "uniffi_xcelerate_checksum_method_page_evaluate_bool": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_bool(),

    "uniffi_xcelerate_checksum_method_page_evaluate_handle": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_handle(),

    "uniffi_xcelerate_checksum_method_page_evaluate_json": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_json(),

    "uniffi_xcelerate_checksum_method_page_evaluate_string": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_string(),

    "uniffi_xcelerate_checksum_method_page_event_names": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_event_names(),

    "uniffi_xcelerate_checksum_method_page_execute_cdp_cmd": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(),

    "uniffi_xcelerate_checksum_method_page_find_element": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_find_element(),

    "uniffi_xcelerate_checksum_method_page_frame": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_frame(),

    "uniffi_xcelerate_checksum_method_page_frame_name": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_frame_name(),

    "uniffi_xcelerate_checksum_method_page_frames": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_frames(),

    "uniffi_xcelerate_checksum_method_page_get_by_label": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_label(),

    "uniffi_xcelerate_checksum_method_page_get_by_role": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_role(),

    "uniffi_xcelerate_checksum_method_page_get_by_text": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_text(),

    "uniffi_xcelerate_checksum_method_page_get_default_timeout": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_get_default_timeout(),

    "uniffi_xcelerate_checksum_method_page_go_back": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_go_back(),

    "uniffi_xcelerate_checksum_method_page_go_forward": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_go_forward(),

    "uniffi_xcelerate_checksum_method_page_handle_js_dialog": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_handle_js_dialog(),

    "uniffi_xcelerate_checksum_method_page_inject_file": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_inject_file(),

    "uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(),

    "uniffi_xcelerate_checksum_method_page_keyboard_down": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_down(),

    "uniffi_xcelerate_checksum_method_page_keyboard_press": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_press(),

    "uniffi_xcelerate_checksum_method_page_keyboard_type": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_type(),

    "uniffi_xcelerate_checksum_method_page_keyboard_up": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_up(),

    "uniffi_xcelerate_checksum_method_page_listens_to": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_listens_to(),

    "uniffi_xcelerate_checksum_method_page_main_frame": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_main_frame(),

    "uniffi_xcelerate_checksum_method_page_metrics": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_metrics(),

    "uniffi_xcelerate_checksum_method_page_mouse_down": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_mouse_down(),

    "uniffi_xcelerate_checksum_method_page_mouse_up": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_mouse_up(),

    "uniffi_xcelerate_checksum_method_page_move_mouse": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_move_mouse(),

    "uniffi_xcelerate_checksum_method_page_navigate": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_navigate(),

    "uniffi_xcelerate_checksum_method_page_on": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_on(),

    "uniffi_xcelerate_checksum_method_page_once": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_once(),

    "uniffi_xcelerate_checksum_method_page_pdf": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_pdf(),

    "uniffi_xcelerate_checksum_method_page_press": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_press(),

    "uniffi_xcelerate_checksum_method_page_query_selector_all": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_query_selector_all(),

    "uniffi_xcelerate_checksum_method_page_query_selector_xpath": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_query_selector_xpath(),

    "uniffi_xcelerate_checksum_method_page_raw_window_bounds": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_raw_window_bounds(),

    "uniffi_xcelerate_checksum_method_page_reload": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_reload(),

    "uniffi_xcelerate_checksum_method_page_remove_all_listeners": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_all_listeners(),

    "uniffi_xcelerate_checksum_method_page_remove_listener": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_listener(),

    "uniffi_xcelerate_checksum_method_page_remove_script": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_script(),

    "uniffi_xcelerate_checksum_method_page_request": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_request(),

    "uniffi_xcelerate_checksum_method_page_requests": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_requests(),

    "uniffi_xcelerate_checksum_method_page_route": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_route(),

    "uniffi_xcelerate_checksum_method_page_route_abort": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_route_abort(),

    "uniffi_xcelerate_checksum_method_page_route_from_har": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_route_from_har(),

    "uniffi_xcelerate_checksum_method_page_route_fulfill": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_route_fulfill(),

    "uniffi_xcelerate_checksum_method_page_screenshot": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_screenshot(),

    "uniffi_xcelerate_checksum_method_page_screenshot_full": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_screenshot_full(),

    "uniffi_xcelerate_checksum_method_page_select_option": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_select_option(),

    "uniffi_xcelerate_checksum_method_page_set_cache_enabled": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_cache_enabled(),

    "uniffi_xcelerate_checksum_method_page_set_content": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_content(),

    "uniffi_xcelerate_checksum_method_page_set_default_timeout": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_default_timeout(),

    "uniffi_xcelerate_checksum_method_page_set_drag_interception": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_drag_interception(),

    "uniffi_xcelerate_checksum_method_page_set_emulated_media_features": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_emulated_media_features(),

    "uniffi_xcelerate_checksum_method_page_set_extra_http_headers": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_extra_http_headers(),

    "uniffi_xcelerate_checksum_method_page_set_input_files": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_input_files(),

    "uniffi_xcelerate_checksum_method_page_set_javascript_enabled": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_javascript_enabled(),

    "uniffi_xcelerate_checksum_method_page_set_offline": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_offline(),

    "uniffi_xcelerate_checksum_method_page_set_request_interception": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_request_interception(),

    "uniffi_xcelerate_checksum_method_page_set_storage_state": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_storage_state(),

    "uniffi_xcelerate_checksum_method_page_set_user_agent": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_user_agent(),

    "uniffi_xcelerate_checksum_method_page_set_viewport_size": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_viewport_size(),

    "uniffi_xcelerate_checksum_method_page_set_window_bounds": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_bounds(),

    "uniffi_xcelerate_checksum_method_page_set_window_position": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_position(),

    "uniffi_xcelerate_checksum_method_page_set_window_size": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_size(),

    "uniffi_xcelerate_checksum_method_page_set_window_state": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_state(),

    "uniffi_xcelerate_checksum_method_page_start_screencast": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_start_screencast(),

    "uniffi_xcelerate_checksum_method_page_start_tracing": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_start_tracing(),

    "uniffi_xcelerate_checksum_method_page_stop_screencast": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_stop_screencast(),

    "uniffi_xcelerate_checksum_method_page_stop_tracing": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_stop_tracing(),

    "uniffi_xcelerate_checksum_method_page_storage_state": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_storage_state(),

    "uniffi_xcelerate_checksum_method_page_target_id": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_target_id(),

    "uniffi_xcelerate_checksum_method_page_title": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_title(),

    "uniffi_xcelerate_checksum_method_page_touch_tap": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_touch_tap(),

    "uniffi_xcelerate_checksum_method_page_unroute": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_unroute(),

    "uniffi_xcelerate_checksum_method_page_unroute_all": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_unroute_all(),

    "uniffi_xcelerate_checksum_method_page_url": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_url(),

    "uniffi_xcelerate_checksum_method_page_wait_for_event": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_event(),

    "uniffi_xcelerate_checksum_method_page_wait_for_event_default": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_event_default(),

    "uniffi_xcelerate_checksum_method_page_wait_for_function": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_function(),

    "uniffi_xcelerate_checksum_method_page_wait_for_navigation": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_navigation(),

    "uniffi_xcelerate_checksum_method_page_wait_for_selector": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_selector(),

    "uniffi_xcelerate_checksum_method_page_wait_for_xpath": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_xpath(),

    "uniffi_xcelerate_checksum_method_page_window_id": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_window_id(),

    "uniffi_xcelerate_checksum_method_page_window_position": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_window_position(),

    "uniffi_xcelerate_checksum_method_page_window_rect": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_window_rect(),

    "uniffi_xcelerate_checksum_method_page_window_size": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_page_window_size(),

    "uniffi_xcelerate_checksum_method_pluginhandle_invoke": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_invoke(),

    "uniffi_xcelerate_checksum_method_pluginhandle_ops": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_ops(),

    "uniffi_xcelerate_checksum_method_pluginhandle_plugin_name": bindings.ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_plugin_name(),

    "uniffi_xcelerate_checksum_constructor_browser_launch": bindings.ffiFunctions.uniffi_xcelerate_checksum_constructor_browser_launch(),

  });
}

export function validateChecksums(bindings = getFfiBindings()) {
  const actualChecksums = getChecksums(bindings);

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_audit_log"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_audit_log"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_audit_log", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_audit_verify"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_audit_verify"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_audit_verify", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_available_plugins"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_available_plugins"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_available_plugins", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_browser_contexts"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_browser_contexts"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_browser_contexts", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_capabilities"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_capabilities"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_capabilities", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_close"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_close"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_close", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_cookies"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_cookies"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_cookies", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_delete_cookie"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_delete_cookie"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_delete_cookie", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_event_names"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_event_names"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_event_names", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_grant_permissions"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_grant_permissions"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_grant_permissions", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_is_connected"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_is_connected"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_is_connected", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_listens_to"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_listens_to"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_listens_to", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_load_plugin"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_load_plugin"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_load_plugin", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_new_context"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_new_context"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_new_context", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_new_page"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_new_page"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_new_page", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_on"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_on"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_on", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_once"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_once"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_once", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_plugin"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_plugin"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_plugin", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_plugin_names"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_plugin_names"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_plugin_names", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_remove_all_listeners"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_remove_all_listeners"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_remove_all_listeners", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_remove_listener"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_remove_listener"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_remove_listener", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_reset_permissions"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_reset_permissions"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_reset_permissions", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_set_cookie"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_set_cookie"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_set_cookie", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_set_download_behavior"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_set_download_behavior"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_set_download_behavior", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_start_tracing"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_start_tracing"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_start_tracing", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_stop_tracing"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_stop_tracing"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_stop_tracing", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_targets"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_targets"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_targets", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_use_plugin"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_use_plugin"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_use_plugin", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_user_agent"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_user_agent"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_user_agent", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_version"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_version"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_version", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_wait_for_event"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_wait_for_event"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_wait_for_event", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_wait_for_event_default"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_wait_for_event_default"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_wait_for_event_default", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_browser_ws_endpoint"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_browser_ws_endpoint"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_browser_ws_endpoint", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_attribute"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_attribute"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_attribute", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_call_bool"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_call_bool"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_call_bool", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_call_json"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_call_json"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_call_json", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_call_on_selector"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_call_on_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_call_on_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_call_on_selector_all"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_call_on_selector_all"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_call_on_selector_all", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_call_string"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_call_string"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_call_string", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_click"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_click"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_click", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_click_mouse"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_click_mouse"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_click_mouse", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_count"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_count"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_count", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_dispose"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_dispose"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_dispose", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_evaluate_bool"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_evaluate_bool"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_evaluate_bool", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_evaluate_handle"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_evaluate_handle"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_evaluate_handle", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_evaluate_json"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_evaluate_json"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_evaluate_json", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_evaluate_string"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_evaluate_string"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_evaluate_string", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_focus"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_focus"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_focus", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_get_by_label"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_get_by_label"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_get_by_label", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_get_by_role"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_get_by_role"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_get_by_role", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_get_by_text"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_get_by_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_get_by_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_get_properties"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_get_properties"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_get_properties", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_hover"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_hover"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_hover", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_hover_mouse"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_hover_mouse"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_hover_mouse", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_inner_html"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_inner_html"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_inner_html", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_press"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_press"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_press", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_query_selector"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_query_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_query_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_query_selector_all"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_query_selector_all"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_query_selector_all", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_query_selector_attr"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_query_selector_attr"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_query_selector_attr", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_query_selector_xpath"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_query_selector_xpath"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_query_selector_xpath", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_screenshot"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_screenshot"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_screenshot", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_screenshot_base64"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_screenshot_base64"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_screenshot_base64", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_select_option"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_select_option"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_select_option", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_set_input_files"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_set_input_files"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_set_input_files", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_text"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_type_text"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_type_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_type_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_element_wait_for_selector"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_element_wait_for_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_element_wait_for_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_activate"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_activate"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_activate", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_activate_target"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_activate_target"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_activate_target", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_add_style_tag"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_add_style_tag"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_add_style_tag", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_authenticate"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_authenticate"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_authenticate", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_bring_to_front"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_bring_to_front"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_bring_to_front", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_call_bool"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_call_bool"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_call_bool", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_call_json"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_call_json"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_call_json", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_call_on_selector"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_call_on_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_call_on_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_call_on_selector_all"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_call_on_selector_all"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_call_on_selector_all", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_call_string"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_call_string"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_call_string", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_clear_requests"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_clear_requests"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_clear_requests", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_click_mouse"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_click_mouse"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_click_mouse", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_close"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_close"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_close", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_content"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_content"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_content", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_cookie"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_cookie"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_cookie", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_cookies"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_cookies"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_cookies", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_coverage_start_css"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_coverage_start_css"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_coverage_start_css", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_coverage_start_js"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_coverage_start_js"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_coverage_start_js", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_coverage_stop_css"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_coverage_stop_css"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_coverage_stop_css", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_coverage_stop_js"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_coverage_stop_js"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_coverage_stop_js", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_create_pdf_stream"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_create_pdf_stream"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_create_pdf_stream", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_decode_base64"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_decode_base64"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_decode_base64", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_default_timeout"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_default_timeout"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_default_timeout", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_document_element"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_document_element"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_document_element", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_emulate_idle_state"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_emulate_idle_state"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_emulate_idle_state", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_emulate_media"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_emulate_media"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_emulate_media", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_ensure_interception"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_ensure_interception"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_ensure_interception", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_evaluate_bool"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_evaluate_bool"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_evaluate_bool", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_evaluate_handle"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_evaluate_handle"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_evaluate_handle", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_evaluate_json"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_evaluate_json"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_evaluate_json", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_evaluate_string"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_evaluate_string"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_evaluate_string", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_event_names"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_event_names"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_event_names", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_execute_cdp_cmd"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_execute_cdp_cmd"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_execute_cdp_cmd", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_find_element"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_find_element"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_find_element", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_frame"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_frame"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_frame", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_frame_name"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_frame_name"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_frame_name", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_frames"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_frames"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_frames", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_get_by_label"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_get_by_label"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_get_by_label", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_get_by_role"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_get_by_role"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_get_by_role", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_get_by_text"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_get_by_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_get_by_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_get_default_timeout"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_get_default_timeout"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_get_default_timeout", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_go_back"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_go_back"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_go_back", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_go_forward"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_go_forward"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_go_forward", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_handle_js_dialog"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_handle_js_dialog"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_handle_js_dialog", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_inject_file"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_inject_file"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_inject_file", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_keyboard_down"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_keyboard_down"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_keyboard_down", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_keyboard_press"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_keyboard_press"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_keyboard_press", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_keyboard_type"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_keyboard_type"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_keyboard_type", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_keyboard_up"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_keyboard_up"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_keyboard_up", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_listens_to"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_listens_to"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_listens_to", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_main_frame"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_main_frame"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_main_frame", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_metrics"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_metrics"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_metrics", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_mouse_down"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_mouse_down"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_mouse_down", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_mouse_up"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_mouse_up"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_mouse_up", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_move_mouse"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_move_mouse"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_move_mouse", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_navigate"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_navigate"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_navigate", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_on"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_on"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_on", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_once"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_once"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_once", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_pdf"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_pdf"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_pdf", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_press"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_press"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_press", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_query_selector_all"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_query_selector_all"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_query_selector_all", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_query_selector_xpath"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_query_selector_xpath"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_query_selector_xpath", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_raw_window_bounds"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_raw_window_bounds"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_raw_window_bounds", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_reload"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_reload"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_reload", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_remove_all_listeners"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_remove_all_listeners"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_remove_all_listeners", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_remove_listener"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_remove_listener"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_remove_listener", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_remove_script"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_remove_script"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_remove_script", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_request"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_request"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_request", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_requests"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_requests"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_requests", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_route"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_route"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_route", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_route_abort"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_route_abort"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_route_abort", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_route_from_har"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_route_from_har"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_route_from_har", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_route_fulfill"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_route_fulfill"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_route_fulfill", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_screenshot"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_screenshot"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_screenshot", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_screenshot_full"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_screenshot_full"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_screenshot_full", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_select_option"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_select_option"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_select_option", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_cache_enabled"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_cache_enabled"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_cache_enabled", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_content"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_content"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_content", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_default_timeout"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_default_timeout"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_default_timeout", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_drag_interception"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_drag_interception"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_drag_interception", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_emulated_media_features"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_emulated_media_features"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_emulated_media_features", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_extra_http_headers"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_extra_http_headers"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_extra_http_headers", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_input_files"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_input_files"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_input_files", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_javascript_enabled"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_javascript_enabled"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_javascript_enabled", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_offline"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_offline"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_offline", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_request_interception"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_request_interception"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_request_interception", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_storage_state"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_storage_state"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_storage_state", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_user_agent"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_user_agent"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_user_agent", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_viewport_size"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_viewport_size"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_viewport_size", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_window_bounds"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_window_bounds"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_window_bounds", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_window_position"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_window_position"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_window_position", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_window_size"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_window_size"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_window_size", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_set_window_state"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_set_window_state"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_set_window_state", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_start_screencast"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_start_screencast"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_start_screencast", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_start_tracing"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_start_tracing"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_start_tracing", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_stop_screencast"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_stop_screencast"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_stop_screencast", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_stop_tracing"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_stop_tracing"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_stop_tracing", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_storage_state"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_storage_state"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_storage_state", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_target_id"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_target_id"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_target_id", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_title"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_title"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_title", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_touch_tap"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_touch_tap"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_touch_tap", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_unroute"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_unroute"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_unroute", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_unroute_all"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_unroute_all"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_unroute_all", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_url"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_url"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_url", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_event"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_event"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_event", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_event_default"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_event_default"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_event_default", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_function"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_function"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_function", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_navigation"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_navigation"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_navigation", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_selector"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_wait_for_xpath"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_wait_for_xpath"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_wait_for_xpath", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_window_id"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_window_id"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_window_id", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_window_position"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_window_position"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_window_position", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_window_rect"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_window_rect"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_window_rect", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_page_window_size"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_page_window_size"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_page_window_size", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_pluginhandle_invoke"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_pluginhandle_invoke"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_pluginhandle_invoke", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_pluginhandle_ops"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_pluginhandle_ops"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_pluginhandle_ops", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_method_pluginhandle_plugin_name"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_method_pluginhandle_plugin_name"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_method_pluginhandle_plugin_name", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_checksum_constructor_browser_launch"];
    const actual = actualChecksums["uniffi_xcelerate_checksum_constructor_browser_launch"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_checksum_constructor_browser_launch", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  return actualChecksums;
}

export const ffiFunctions = Object.freeze({

  uniffi_xcelerate_fn_clone_browser(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_browser(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_browser_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_browser_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_free_browser(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_browser(...args);

    return result;

  },


  uniffi_xcelerate_fn_free_browser_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_browser_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_fn_constructor_browser_launch(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_constructor_browser_launch(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_audit_log(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_audit_log(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_audit_log_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_audit_log_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_audit_verify(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_audit_verify(...args);

    return result;

  },


  uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_fn_method_browser_available_plugins(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_available_plugins(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_browser_contexts(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_browser_contexts(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_capabilities(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_capabilities(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_capabilities_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_capabilities_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_close(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_close_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_close_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_cookies(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_cookies(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_cookies_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_cookies_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_delete_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_delete_cookie(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_event_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_event_names(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_event_names_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_event_names_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_grant_permissions(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_grant_permissions(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_is_connected(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_is_connected(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_is_connected_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_is_connected_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_listens_to(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_listens_to(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_listens_to_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_listens_to_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_load_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_load_plugin(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_new_context(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_new_context(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_new_context_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_new_context_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_new_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_new_page(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_new_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_new_page_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_on(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_on(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_on_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_on_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_once(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_once(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_once_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_once_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_plugin(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_plugin_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_plugin_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_plugin_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_plugin_names(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_remove_all_listeners(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_remove_all_listeners(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_remove_listener(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_remove_listener(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_reset_permissions(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_reset_permissions(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_set_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_set_cookie(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_set_download_behavior(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_set_download_behavior(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_start_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_start_tracing(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_stop_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_stop_tracing(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_targets(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_targets(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_targets_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_targets_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_use_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_use_plugin(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_user_agent(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_user_agent(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_user_agent_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_user_agent_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_version(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_version(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_version_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_version_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_wait_for_event(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_wait_for_event(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_wait_for_event_default(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_wait_for_event_default(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_browser_ws_endpoint(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_ws_endpoint(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_clone_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_element(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_element_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_free_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_element(...args);

    return result;

  },


  uniffi_xcelerate_fn_free_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_element_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_fn_method_element_attribute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_attribute(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_attribute_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_attribute_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_bool(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_bool_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_bool_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_json(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_json_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_json_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_on_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_on_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_on_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_on_selector_all(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_string(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_call_string_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_call_string_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_click(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_click(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_click_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_click_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_click_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_click_mouse(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_click_mouse_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_click_mouse_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_count(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_count(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_count_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_count_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_dispose(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_dispose(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_dispose_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_dispose_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_bool(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_handle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_handle(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_json(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_string(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_focus(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_focus(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_focus_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_focus_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_label(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_label(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_label_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_label_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_role(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_role(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_role_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_role_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_by_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_by_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_properties(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_properties(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_get_properties_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_get_properties_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_hover(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_hover(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_hover_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_hover_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_hover_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_hover_mouse(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_hover_mouse_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_hover_mouse_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_inner_html(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_inner_html(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_inner_html_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_inner_html_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_press(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_press_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_press_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_all(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_attr(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_attr(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_xpath(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_screenshot(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_screenshot_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_screenshot_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_screenshot_base64(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_screenshot_base64(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_select_option(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_select_option(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_select_option_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_select_option_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_set_input_files(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_set_input_files(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_set_input_files_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_set_input_files_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_type_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_type_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_type_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_type_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_wait_for_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_page(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_page_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_free_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_page(...args);

    return result;

  },


  uniffi_xcelerate_fn_free_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_page_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_fn_method_page_activate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_activate(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_activate_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_activate_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_activate_target(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_activate_target(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_activate_target_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_activate_target_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_add_style_tag(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_add_style_tag(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_authenticate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_authenticate(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_authenticate_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_authenticate_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_bring_to_front(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_bring_to_front(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_bool(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_bool_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_bool_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_json(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_json_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_json_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_on_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_on_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_on_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_on_selector_all(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_string(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_call_string_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_call_string_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_clear_requests(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_clear_requests(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_clear_requests_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_clear_requests_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_click_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_click_mouse(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_click_mouse_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_click_mouse_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_close(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_close_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_close_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_content(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_content_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_content_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_cookie(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_cookie_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_cookie_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_cookies(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_cookies(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_cookies_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_cookies_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_start_css(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_start_css(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_start_js(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_start_js(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_stop_css(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_stop_css(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_stop_js(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_stop_js(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_create_pdf_stream(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_create_pdf_stream(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_decode_base64(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_decode_base64(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_page_decode_base64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_decode_base64_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_page_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_default_timeout(...args);

    return normalizeUInt64(result);

  },


  uniffi_xcelerate_fn_method_page_default_timeout_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_default_timeout_generic_abi(...args);

    return normalizeUInt64(result);

  },


  uniffi_xcelerate_fn_method_page_document_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_document_element(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_document_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_document_element_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_emulate_idle_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_emulate_idle_state(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_emulate_media(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_emulate_media(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_emulate_media_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_emulate_media_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_ensure_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_ensure_interception(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_bool(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_handle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_handle(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_json(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_string(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_event_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_event_names(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_event_names_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_event_names_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_execute_cdp_cmd(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_execute_cdp_cmd(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_find_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_find_element(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_find_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_find_element_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frame(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frame(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frame_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frame_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frame_name(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frame_name(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frame_name_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frame_name_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frames(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frames(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_frames_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_frames_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_label(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_label(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_label_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_label_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_role(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_role(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_role_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_role_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_by_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_by_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_default_timeout(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_go_back(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_go_back(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_go_back_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_go_back_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_go_forward(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_go_forward(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_go_forward_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_go_forward_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_handle_js_dialog(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_handle_js_dialog(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_inject_file(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_inject_file(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_inject_file_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_inject_file_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_down(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_down(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_press(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_type(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_type(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_up(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_up(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_listens_to(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_listens_to(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_listens_to_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_listens_to_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_main_frame(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_main_frame(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_main_frame_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_main_frame_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_metrics(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_metrics(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_metrics_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_metrics_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_mouse_down(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_mouse_down(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_mouse_down_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_mouse_down_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_mouse_up(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_mouse_up(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_mouse_up_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_mouse_up_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_move_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_move_mouse(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_move_mouse_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_move_mouse_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_navigate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_navigate(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_navigate_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_navigate_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_on(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_on(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_on_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_on_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_once(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_once(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_once_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_once_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_pdf(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_pdf(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_pdf_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_pdf_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_press(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_press_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_press_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_query_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_query_selector_all(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_query_selector_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_query_selector_xpath(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_raw_window_bounds(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_raw_window_bounds(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_reload(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_reload(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_reload_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_reload_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_all_listeners(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_all_listeners(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_listener(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_listener(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_listener_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_listener_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_script(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_script(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_remove_script_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_remove_script_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_request(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_request(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_request_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_request_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_requests(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_requests(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_requests_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_requests_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_abort(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_abort(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_abort_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_abort_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_from_har(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_from_har(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_from_har_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_from_har_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_fulfill(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_fulfill(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_screenshot(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_screenshot_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_screenshot_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_screenshot_full(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_screenshot_full(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_select_option(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_select_option(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_select_option_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_select_option_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_cache_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_cache_enabled(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_content(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_content_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_content_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_default_timeout(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_drag_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_drag_interception(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_emulated_media_features(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_emulated_media_features(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_extra_http_headers(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_extra_http_headers(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_input_files(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_input_files(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_input_files_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_input_files_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_javascript_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_javascript_enabled(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_offline(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_offline(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_offline_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_offline_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_request_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_request_interception(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_storage_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_storage_state(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_user_agent(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_user_agent(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_viewport_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_viewport_size(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_bounds(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_bounds(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_position(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_position(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_position_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_position_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_size(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_size_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_size_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_state(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_set_window_state_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_set_window_state_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_start_screencast(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_start_screencast(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_start_screencast_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_start_screencast_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_start_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_start_tracing(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_start_tracing_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_start_tracing_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_stop_screencast(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_stop_screencast(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_stop_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_stop_tracing(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_storage_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_storage_state(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_storage_state_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_storage_state_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_target_id(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_target_id(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_page_target_id_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_target_id_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_page_title(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_title(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_title_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_title_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_touch_tap(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_touch_tap(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_touch_tap_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_touch_tap_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_unroute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_unroute(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_unroute_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_unroute_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_unroute_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_unroute_all(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_unroute_all_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_unroute_all_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_url(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_url(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_url_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_url_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_event(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_event(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_event_default(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_event_default(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_function(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_function(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_navigation(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_navigation(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_xpath(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_id(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_id(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_id_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_id_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_position(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_position(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_position_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_position_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_rect(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_rect(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_rect_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_rect_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_size(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_page_window_size_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_page_window_size_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_pluginhandle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_pluginhandle(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_clone_pluginhandle_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_clone_pluginhandle_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_free_pluginhandle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_pluginhandle(...args);

    return result;

  },


  uniffi_xcelerate_fn_free_pluginhandle_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_free_pluginhandle_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_fn_method_pluginhandle_invoke(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_invoke(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_fn_method_pluginhandle_ops(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_ops(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_pluginhandle_plugin_name(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_plugin_name(...args);

    return normalizeRustBuffer(result);

  },


  uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rustbuffer_alloc(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rustbuffer_alloc(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rustbuffer_from_bytes(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rustbuffer_from_bytes(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rustbuffer_free(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rustbuffer_free(...args);

    return result;

  },


  ffi_xcelerate_rustbuffer_reserve(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rustbuffer_reserve(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rust_future_poll_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i8(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i16(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u64(...args);

    return normalizeUInt64(result);

  },


  ffi_xcelerate_rust_future_complete_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_u64_generic_abi(...args);

    return normalizeUInt64(result);

  },


  ffi_xcelerate_rust_future_poll_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i64(...args);

    return normalizeInt64(result);

  },


  ffi_xcelerate_rust_future_complete_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_i64_generic_abi(...args);

    return normalizeInt64(result);

  },


  ffi_xcelerate_rust_future_poll_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_f32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_f32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_f32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_f32(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_f64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_f64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_f64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_f64(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_rust_buffer(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rust_future_complete_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_rust_buffer_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_rust_future_poll_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_void(...args);

    return result;

  },


  ffi_xcelerate_rust_future_poll_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_poll_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_void(...args);

    return result;

  },


  ffi_xcelerate_rust_future_cancel_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_cancel_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_void(...args);

    return result;

  },


  ffi_xcelerate_rust_future_free_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_free_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_void(...args);

    return result;

  },


  ffi_xcelerate_rust_future_complete_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_rust_future_complete_void_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_audit_log(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_audit_log(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_audit_verify(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_audit_verify(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_available_plugins(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_available_plugins(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_browser_contexts(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_browser_contexts(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_capabilities(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_capabilities(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_close(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_cookies(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_cookies(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_delete_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_delete_cookie(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_event_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_event_names(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_grant_permissions(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_grant_permissions(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_is_connected(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_is_connected(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_listens_to(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_listens_to(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_load_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_load_plugin(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_new_context(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_new_context(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_new_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_new_page(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_on(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_on(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_once(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_once(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_plugin(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_plugin_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_plugin_names(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_remove_all_listeners(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_remove_all_listeners(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_remove_listener(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_remove_listener(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_reset_permissions(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_reset_permissions(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_set_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_set_cookie(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_set_download_behavior(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_set_download_behavior(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_start_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_start_tracing(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_stop_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_stop_tracing(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_targets(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_targets(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_use_plugin(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_use_plugin(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_user_agent(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_user_agent(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_version(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_version(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_wait_for_event(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_wait_for_event(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_wait_for_event_default(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_wait_for_event_default(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_browser_ws_endpoint(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_browser_ws_endpoint(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_attribute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_attribute(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_call_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_call_bool(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_call_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_call_json(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_call_on_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_call_on_selector(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_call_on_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_call_on_selector_all(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_call_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_call_string(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_click(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_click(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_click_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_click_mouse(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_count(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_count(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_dispose(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_dispose(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_evaluate_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_evaluate_bool(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_evaluate_handle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_evaluate_handle(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_evaluate_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_evaluate_json(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_evaluate_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_evaluate_string(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_focus(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_focus(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_get_by_label(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_get_by_label(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_get_by_role(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_get_by_role(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_get_by_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_get_by_text(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_get_properties(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_get_properties(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_hover(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_hover(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_hover_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_hover_mouse(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_inner_html(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_inner_html(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_press(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_query_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_query_selector(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_query_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_query_selector_all(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_query_selector_attr(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_query_selector_attr(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_query_selector_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_query_selector_xpath(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_screenshot(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_screenshot_base64(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_screenshot_base64(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_select_option(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_select_option(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_set_input_files(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_set_input_files(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_text(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_type_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_type_text(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_element_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_element_wait_for_selector(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_activate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_activate(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_activate_target(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_activate_target(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_add_style_tag(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_add_style_tag(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_authenticate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_authenticate(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_bring_to_front(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_bring_to_front(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_call_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_call_bool(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_call_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_call_json(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_call_on_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_call_on_selector(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_call_on_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_call_on_selector_all(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_call_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_call_string(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_clear_requests(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_clear_requests(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_click_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_click_mouse(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_close(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_content(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_cookie(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_cookie(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_cookies(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_cookies(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_coverage_start_css(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_coverage_start_css(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_coverage_start_js(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_coverage_start_js(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_coverage_stop_css(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_coverage_stop_css(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_coverage_stop_js(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_coverage_stop_js(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_create_pdf_stream(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_create_pdf_stream(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_decode_base64(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_decode_base64(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_default_timeout(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_document_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_document_element(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_emulate_idle_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_emulate_idle_state(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_emulate_media(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_emulate_media(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_ensure_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_ensure_interception(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_evaluate_bool(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_evaluate_bool(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_evaluate_handle(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_evaluate_handle(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_evaluate_json(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_evaluate_json(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_evaluate_string(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_evaluate_string(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_event_names(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_event_names(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_find_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_find_element(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_frame(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_frame(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_frame_name(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_frame_name(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_frames(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_frames(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_get_by_label(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_get_by_label(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_get_by_role(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_get_by_role(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_get_by_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_get_by_text(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_get_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_get_default_timeout(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_go_back(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_go_back(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_go_forward(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_go_forward(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_handle_js_dialog(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_handle_js_dialog(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_inject_file(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_inject_file(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_keyboard_down(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_keyboard_down(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_keyboard_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_keyboard_press(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_keyboard_type(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_keyboard_type(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_keyboard_up(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_keyboard_up(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_listens_to(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_listens_to(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_main_frame(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_main_frame(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_metrics(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_metrics(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_mouse_down(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_mouse_down(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_mouse_up(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_mouse_up(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_move_mouse(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_move_mouse(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_navigate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_navigate(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_on(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_on(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_once(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_once(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_pdf(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_pdf(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_press(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_press(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_query_selector_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_query_selector_all(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_query_selector_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_query_selector_xpath(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_raw_window_bounds(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_raw_window_bounds(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_reload(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_reload(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_remove_all_listeners(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_remove_all_listeners(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_remove_listener(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_remove_listener(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_remove_script(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_remove_script(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_request(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_request(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_requests(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_requests(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_route(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_route(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_route_abort(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_route_abort(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_route_from_har(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_route_from_har(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_route_fulfill(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_route_fulfill(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_screenshot(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_screenshot_full(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_screenshot_full(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_select_option(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_select_option(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_cache_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_cache_enabled(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_content(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_default_timeout(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_default_timeout(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_drag_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_drag_interception(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_emulated_media_features(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_emulated_media_features(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_extra_http_headers(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_extra_http_headers(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_input_files(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_input_files(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_javascript_enabled(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_javascript_enabled(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_offline(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_offline(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_request_interception(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_request_interception(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_storage_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_storage_state(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_user_agent(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_user_agent(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_viewport_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_viewport_size(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_window_bounds(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_window_bounds(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_window_position(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_window_position(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_window_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_window_size(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_set_window_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_set_window_state(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_start_screencast(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_start_screencast(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_start_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_start_tracing(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_stop_screencast(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_stop_screencast(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_stop_tracing(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_stop_tracing(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_storage_state(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_storage_state(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_target_id(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_target_id(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_title(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_title(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_touch_tap(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_touch_tap(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_unroute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_unroute(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_unroute_all(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_unroute_all(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_url(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_url(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_event(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_event(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_event_default(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_event_default(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_function(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_function(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_navigation(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_navigation(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_selector(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_wait_for_xpath(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_wait_for_xpath(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_window_id(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_window_id(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_window_position(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_window_position(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_window_rect(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_window_rect(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_page_window_size(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_page_window_size(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_pluginhandle_invoke(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_pluginhandle_invoke(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_pluginhandle_ops(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_pluginhandle_ops(...args);

    return result;

  },


  uniffi_xcelerate_checksum_method_pluginhandle_plugin_name(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_method_pluginhandle_plugin_name(...args);

    return result;

  },


  uniffi_xcelerate_checksum_constructor_browser_launch(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_checksum_constructor_browser_launch(...args);

    return result;

  },


  ffi_xcelerate_uniffi_contract_version(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_uniffi_contract_version(...args);

    return result;

  },


});


export function uniffi_xcelerate_fn_clone_browser(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_browser(...args);
}


export function uniffi_xcelerate_fn_clone_browser_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_browser_generic_abi(...args);
}



export function uniffi_xcelerate_fn_free_browser(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_browser(...args);
}


export function uniffi_xcelerate_fn_free_browser_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_browser_generic_abi(...args);
}



export function uniffi_xcelerate_fn_constructor_browser_launch(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_constructor_browser_launch(...args);
}



export function uniffi_xcelerate_fn_method_browser_audit_log(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_log(...args);
}


export function uniffi_xcelerate_fn_method_browser_audit_log_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_log_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_audit_verify(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_verify(...args);
}


export function uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_available_plugins(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_available_plugins(...args);
}


export function uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_browser_contexts(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_browser_contexts(...args);
}


export function uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_capabilities(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_capabilities(...args);
}


export function uniffi_xcelerate_fn_method_browser_capabilities_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_capabilities_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_close(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_close(...args);
}


export function uniffi_xcelerate_fn_method_browser_close_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_close_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_cookies(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_cookies(...args);
}


export function uniffi_xcelerate_fn_method_browser_cookies_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_cookies_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_delete_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_delete_cookie(...args);
}


export function uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_event_names(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_event_names(...args);
}


export function uniffi_xcelerate_fn_method_browser_event_names_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_event_names_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_grant_permissions(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_grant_permissions(...args);
}


export function uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_is_connected(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_is_connected(...args);
}


export function uniffi_xcelerate_fn_method_browser_is_connected_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_is_connected_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_listens_to(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_listens_to(...args);
}


export function uniffi_xcelerate_fn_method_browser_listens_to_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_listens_to_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_load_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_load_plugin(...args);
}


export function uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_new_context(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_new_context(...args);
}


export function uniffi_xcelerate_fn_method_browser_new_context_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_new_context_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_new_page(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_new_page(...args);
}


export function uniffi_xcelerate_fn_method_browser_new_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_new_page_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_on(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_on(...args);
}


export function uniffi_xcelerate_fn_method_browser_on_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_on_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_once(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_once(...args);
}


export function uniffi_xcelerate_fn_method_browser_once_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_once_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin(...args);
}


export function uniffi_xcelerate_fn_method_browser_plugin_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_plugin_names(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_names(...args);
}


export function uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_remove_all_listeners(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_all_listeners(...args);
}


export function uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_remove_listener(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_listener(...args);
}


export function uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_reset_permissions(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_reset_permissions(...args);
}


export function uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_set_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_set_cookie(...args);
}


export function uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_set_download_behavior(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_set_download_behavior(...args);
}


export function uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_start_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_start_tracing(...args);
}


export function uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_stop_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_stop_tracing(...args);
}


export function uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_targets(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_targets(...args);
}


export function uniffi_xcelerate_fn_method_browser_targets_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_targets_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_use_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_use_plugin(...args);
}


export function uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_user_agent(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_user_agent(...args);
}


export function uniffi_xcelerate_fn_method_browser_user_agent_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_user_agent_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_version(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_version(...args);
}


export function uniffi_xcelerate_fn_method_browser_version_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_version_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_wait_for_event(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event(...args);
}


export function uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_wait_for_event_default(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_default(...args);
}


export function uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_browser_ws_endpoint(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_ws_endpoint(...args);
}


export function uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi(...args);
}



export function uniffi_xcelerate_fn_clone_element(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_element(...args);
}


export function uniffi_xcelerate_fn_clone_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_element_generic_abi(...args);
}



export function uniffi_xcelerate_fn_free_element(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_element(...args);
}


export function uniffi_xcelerate_fn_free_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_element_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_attribute(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_attribute(...args);
}


export function uniffi_xcelerate_fn_method_element_attribute_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_attribute_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_call_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_bool(...args);
}


export function uniffi_xcelerate_fn_method_element_call_bool_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_bool_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_call_json(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_json(...args);
}


export function uniffi_xcelerate_fn_method_element_call_json_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_json_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_call_on_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector(...args);
}


export function uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_call_on_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_all(...args);
}


export function uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_call_string(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_string(...args);
}


export function uniffi_xcelerate_fn_method_element_call_string_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_call_string_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_click(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_click(...args);
}


export function uniffi_xcelerate_fn_method_element_click_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_click_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_click_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_click_mouse(...args);
}


export function uniffi_xcelerate_fn_method_element_click_mouse_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_click_mouse_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_count(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_count(...args);
}


export function uniffi_xcelerate_fn_method_element_count_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_count_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_dispose(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_dispose(...args);
}


export function uniffi_xcelerate_fn_method_element_dispose_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_dispose_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_evaluate_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_bool(...args);
}


export function uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_evaluate_handle(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_handle(...args);
}


export function uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_evaluate_json(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_json(...args);
}


export function uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_evaluate_string(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_string(...args);
}


export function uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_focus(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_focus(...args);
}


export function uniffi_xcelerate_fn_method_element_focus_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_focus_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_get_by_label(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_label(...args);
}


export function uniffi_xcelerate_fn_method_element_get_by_label_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_label_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_get_by_role(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_role(...args);
}


export function uniffi_xcelerate_fn_method_element_get_by_role_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_role_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_get_by_text(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_text(...args);
}


export function uniffi_xcelerate_fn_method_element_get_by_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_text_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_get_properties(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_properties(...args);
}


export function uniffi_xcelerate_fn_method_element_get_properties_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_get_properties_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_hover(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_hover(...args);
}


export function uniffi_xcelerate_fn_method_element_hover_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_hover_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_hover_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_hover_mouse(...args);
}


export function uniffi_xcelerate_fn_method_element_hover_mouse_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_hover_mouse_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_inner_html(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_inner_html(...args);
}


export function uniffi_xcelerate_fn_method_element_inner_html_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_inner_html_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_press(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_press(...args);
}


export function uniffi_xcelerate_fn_method_element_press_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_press_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_query_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector(...args);
}


export function uniffi_xcelerate_fn_method_element_query_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_query_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_all(...args);
}


export function uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_query_selector_attr(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_attr(...args);
}


export function uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_query_selector_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_xpath(...args);
}


export function uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot(...args);
}


export function uniffi_xcelerate_fn_method_element_screenshot_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_screenshot_base64(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_base64(...args);
}


export function uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_select_option(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_select_option(...args);
}


export function uniffi_xcelerate_fn_method_element_select_option_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_select_option_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_set_input_files(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_set_input_files(...args);
}


export function uniffi_xcelerate_fn_method_element_set_input_files_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_set_input_files_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_text(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_text(...args);
}


export function uniffi_xcelerate_fn_method_element_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_text_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_type_text(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_type_text(...args);
}


export function uniffi_xcelerate_fn_method_element_type_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_type_text_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_element_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_wait_for_selector(...args);
}


export function uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi(...args);
}



export function uniffi_xcelerate_fn_clone_page(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_page(...args);
}


export function uniffi_xcelerate_fn_clone_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_page_generic_abi(...args);
}



export function uniffi_xcelerate_fn_free_page(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_page(...args);
}


export function uniffi_xcelerate_fn_free_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_page_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_activate(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_activate(...args);
}


export function uniffi_xcelerate_fn_method_page_activate_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_activate_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_activate_target(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_activate_target(...args);
}


export function uniffi_xcelerate_fn_method_page_activate_target_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_activate_target_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(...args);
}


export function uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_add_style_tag(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_add_style_tag(...args);
}


export function uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_authenticate(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_authenticate(...args);
}


export function uniffi_xcelerate_fn_method_page_authenticate_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_authenticate_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_bring_to_front(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_bring_to_front(...args);
}


export function uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_call_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_bool(...args);
}


export function uniffi_xcelerate_fn_method_page_call_bool_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_bool_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_call_json(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_json(...args);
}


export function uniffi_xcelerate_fn_method_page_call_json_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_json_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_call_on_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector(...args);
}


export function uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_call_on_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_all(...args);
}


export function uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_call_string(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_string(...args);
}


export function uniffi_xcelerate_fn_method_page_call_string_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_call_string_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_clear_requests(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_clear_requests(...args);
}


export function uniffi_xcelerate_fn_method_page_clear_requests_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_clear_requests_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_click_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_click_mouse(...args);
}


export function uniffi_xcelerate_fn_method_page_click_mouse_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_click_mouse_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_close(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_close(...args);
}


export function uniffi_xcelerate_fn_method_page_close_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_close_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_content(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_content(...args);
}


export function uniffi_xcelerate_fn_method_page_content_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_content_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_cookie(...args);
}


export function uniffi_xcelerate_fn_method_page_cookie_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_cookie_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_cookies(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_cookies(...args);
}


export function uniffi_xcelerate_fn_method_page_cookies_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_cookies_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_coverage_start_css(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_css(...args);
}


export function uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_coverage_start_js(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_js(...args);
}


export function uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_coverage_stop_css(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_css(...args);
}


export function uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_coverage_stop_js(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_js(...args);
}


export function uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_create_pdf_stream(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_create_pdf_stream(...args);
}


export function uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_decode_base64(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_decode_base64(...args);
}


export function uniffi_xcelerate_fn_method_page_decode_base64_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_decode_base64_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_default_timeout(...args);
}


export function uniffi_xcelerate_fn_method_page_default_timeout_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_default_timeout_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_document_element(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_document_element(...args);
}


export function uniffi_xcelerate_fn_method_page_document_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_document_element_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_emulate_idle_state(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_idle_state(...args);
}


export function uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_emulate_media(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_media(...args);
}


export function uniffi_xcelerate_fn_method_page_emulate_media_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_media_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_ensure_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_ensure_interception(...args);
}


export function uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_evaluate_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_bool(...args);
}


export function uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_evaluate_handle(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_handle(...args);
}


export function uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_evaluate_json(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_json(...args);
}


export function uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_evaluate_string(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_string(...args);
}


export function uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_event_names(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_event_names(...args);
}


export function uniffi_xcelerate_fn_method_page_event_names_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_event_names_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_execute_cdp_cmd(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_execute_cdp_cmd(...args);
}


export function uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_find_element(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_find_element(...args);
}


export function uniffi_xcelerate_fn_method_page_find_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_find_element_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_frame(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frame(...args);
}


export function uniffi_xcelerate_fn_method_page_frame_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frame_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_frame_name(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frame_name(...args);
}


export function uniffi_xcelerate_fn_method_page_frame_name_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frame_name_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_frames(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frames(...args);
}


export function uniffi_xcelerate_fn_method_page_frames_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_frames_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_get_by_label(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_label(...args);
}


export function uniffi_xcelerate_fn_method_page_get_by_label_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_label_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_get_by_role(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_role(...args);
}


export function uniffi_xcelerate_fn_method_page_get_by_role_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_role_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_get_by_text(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_text(...args);
}


export function uniffi_xcelerate_fn_method_page_get_by_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_text_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_get_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_default_timeout(...args);
}


export function uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_go_back(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_go_back(...args);
}


export function uniffi_xcelerate_fn_method_page_go_back_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_go_back_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_go_forward(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_go_forward(...args);
}


export function uniffi_xcelerate_fn_method_page_go_forward_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_go_forward_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_handle_js_dialog(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_handle_js_dialog(...args);
}


export function uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_inject_file(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_inject_file(...args);
}


export function uniffi_xcelerate_fn_method_page_inject_file_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_inject_file_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(...args);
}


export function uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_keyboard_down(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_down(...args);
}


export function uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_keyboard_press(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_press(...args);
}


export function uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_keyboard_type(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_type(...args);
}


export function uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_keyboard_up(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_up(...args);
}


export function uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_listens_to(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_listens_to(...args);
}


export function uniffi_xcelerate_fn_method_page_listens_to_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_listens_to_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_main_frame(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_main_frame(...args);
}


export function uniffi_xcelerate_fn_method_page_main_frame_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_main_frame_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_metrics(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_metrics(...args);
}


export function uniffi_xcelerate_fn_method_page_metrics_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_metrics_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_mouse_down(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_down(...args);
}


export function uniffi_xcelerate_fn_method_page_mouse_down_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_down_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_mouse_up(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_up(...args);
}


export function uniffi_xcelerate_fn_method_page_mouse_up_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_up_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_move_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_move_mouse(...args);
}


export function uniffi_xcelerate_fn_method_page_move_mouse_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_move_mouse_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_navigate(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_navigate(...args);
}


export function uniffi_xcelerate_fn_method_page_navigate_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_navigate_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_on(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_on(...args);
}


export function uniffi_xcelerate_fn_method_page_on_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_on_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_once(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_once(...args);
}


export function uniffi_xcelerate_fn_method_page_once_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_once_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_pdf(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_pdf(...args);
}


export function uniffi_xcelerate_fn_method_page_pdf_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_pdf_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_press(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_press(...args);
}


export function uniffi_xcelerate_fn_method_page_press_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_press_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_query_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_all(...args);
}


export function uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_query_selector_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_xpath(...args);
}


export function uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_raw_window_bounds(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_raw_window_bounds(...args);
}


export function uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_reload(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_reload(...args);
}


export function uniffi_xcelerate_fn_method_page_reload_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_reload_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_remove_all_listeners(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_all_listeners(...args);
}


export function uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_remove_listener(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_listener(...args);
}


export function uniffi_xcelerate_fn_method_page_remove_listener_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_listener_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_remove_script(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_script(...args);
}


export function uniffi_xcelerate_fn_method_page_remove_script_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_remove_script_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_request(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_request(...args);
}


export function uniffi_xcelerate_fn_method_page_request_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_request_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_requests(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_requests(...args);
}


export function uniffi_xcelerate_fn_method_page_requests_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_requests_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_route(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route(...args);
}


export function uniffi_xcelerate_fn_method_page_route_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_route_abort(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_abort(...args);
}


export function uniffi_xcelerate_fn_method_page_route_abort_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_abort_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_route_from_har(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_from_har(...args);
}


export function uniffi_xcelerate_fn_method_page_route_from_har_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_from_har_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_route_fulfill(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_fulfill(...args);
}


export function uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot(...args);
}


export function uniffi_xcelerate_fn_method_page_screenshot_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_screenshot_full(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_full(...args);
}


export function uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_select_option(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_select_option(...args);
}


export function uniffi_xcelerate_fn_method_page_select_option_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_select_option_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_cache_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_cache_enabled(...args);
}


export function uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_content(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_content(...args);
}


export function uniffi_xcelerate_fn_method_page_set_content_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_content_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_default_timeout(...args);
}


export function uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_drag_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_drag_interception(...args);
}


export function uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_emulated_media_features(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_emulated_media_features(...args);
}


export function uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_extra_http_headers(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_extra_http_headers(...args);
}


export function uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_input_files(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_input_files(...args);
}


export function uniffi_xcelerate_fn_method_page_set_input_files_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_input_files_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_javascript_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_javascript_enabled(...args);
}


export function uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_offline(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_offline(...args);
}


export function uniffi_xcelerate_fn_method_page_set_offline_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_offline_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_request_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_request_interception(...args);
}


export function uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_storage_state(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_storage_state(...args);
}


export function uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_user_agent(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_user_agent(...args);
}


export function uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_viewport_size(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_viewport_size(...args);
}


export function uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_window_bounds(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_bounds(...args);
}


export function uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_window_position(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_position(...args);
}


export function uniffi_xcelerate_fn_method_page_set_window_position_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_position_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_window_size(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_size(...args);
}


export function uniffi_xcelerate_fn_method_page_set_window_size_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_size_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_set_window_state(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_state(...args);
}


export function uniffi_xcelerate_fn_method_page_set_window_state_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_state_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_start_screencast(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_start_screencast(...args);
}


export function uniffi_xcelerate_fn_method_page_start_screencast_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_start_screencast_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_start_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_start_tracing(...args);
}


export function uniffi_xcelerate_fn_method_page_start_tracing_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_start_tracing_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_stop_screencast(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_stop_screencast(...args);
}


export function uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_stop_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_stop_tracing(...args);
}


export function uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_storage_state(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_storage_state(...args);
}


export function uniffi_xcelerate_fn_method_page_storage_state_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_storage_state_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_target_id(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_target_id(...args);
}


export function uniffi_xcelerate_fn_method_page_target_id_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_target_id_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_title(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_title(...args);
}


export function uniffi_xcelerate_fn_method_page_title_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_title_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_touch_tap(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_touch_tap(...args);
}


export function uniffi_xcelerate_fn_method_page_touch_tap_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_touch_tap_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_unroute(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_unroute(...args);
}


export function uniffi_xcelerate_fn_method_page_unroute_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_unroute_all(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_all(...args);
}


export function uniffi_xcelerate_fn_method_page_unroute_all_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_all_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_url(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_url(...args);
}


export function uniffi_xcelerate_fn_method_page_url_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_url_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_event(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_event_default(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_default(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_function(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_function(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_navigation(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_navigation(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_selector(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_wait_for_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_xpath(...args);
}


export function uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_window_id(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_id(...args);
}


export function uniffi_xcelerate_fn_method_page_window_id_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_id_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_window_position(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_position(...args);
}


export function uniffi_xcelerate_fn_method_page_window_position_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_position_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_window_rect(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_rect(...args);
}


export function uniffi_xcelerate_fn_method_page_window_rect_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_rect_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_page_window_size(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_size(...args);
}


export function uniffi_xcelerate_fn_method_page_window_size_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_page_window_size_generic_abi(...args);
}



export function uniffi_xcelerate_fn_clone_pluginhandle(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_pluginhandle(...args);
}


export function uniffi_xcelerate_fn_clone_pluginhandle_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_clone_pluginhandle_generic_abi(...args);
}



export function uniffi_xcelerate_fn_free_pluginhandle(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_pluginhandle(...args);
}


export function uniffi_xcelerate_fn_free_pluginhandle_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_free_pluginhandle_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_pluginhandle_invoke(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_invoke(...args);
}


export function uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_pluginhandle_ops(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_ops(...args);
}


export function uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi(...args);
}



export function uniffi_xcelerate_fn_method_pluginhandle_plugin_name(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_plugin_name(...args);
}


export function uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi(...args);
}



export function ffi_xcelerate_rustbuffer_alloc(...args) {
  return ffiFunctions.ffi_xcelerate_rustbuffer_alloc(...args);
}



export function ffi_xcelerate_rustbuffer_from_bytes(...args) {
  return ffiFunctions.ffi_xcelerate_rustbuffer_from_bytes(...args);
}



export function ffi_xcelerate_rustbuffer_free(...args) {
  return ffiFunctions.ffi_xcelerate_rustbuffer_free(...args);
}



export function ffi_xcelerate_rustbuffer_reserve(...args) {
  return ffiFunctions.ffi_xcelerate_rustbuffer_reserve(...args);
}



export function ffi_xcelerate_rust_future_poll_u8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u8(...args);
}


export function ffi_xcelerate_rust_future_poll_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_u8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u8(...args);
}


export function ffi_xcelerate_rust_future_cancel_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_u8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u8(...args);
}


export function ffi_xcelerate_rust_future_free_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_u8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u8(...args);
}


export function ffi_xcelerate_rust_future_complete_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_i8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i8(...args);
}


export function ffi_xcelerate_rust_future_poll_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_i8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(...args);
}


export function ffi_xcelerate_rust_future_cancel_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_i8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i8(...args);
}


export function ffi_xcelerate_rust_future_free_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_i8(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i8(...args);
}


export function ffi_xcelerate_rust_future_complete_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i8_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_u16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u16(...args);
}


export function ffi_xcelerate_rust_future_poll_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_u16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u16(...args);
}


export function ffi_xcelerate_rust_future_cancel_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_u16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u16(...args);
}


export function ffi_xcelerate_rust_future_free_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_u16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u16(...args);
}


export function ffi_xcelerate_rust_future_complete_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_i16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i16(...args);
}


export function ffi_xcelerate_rust_future_poll_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_i16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i16(...args);
}


export function ffi_xcelerate_rust_future_cancel_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_i16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i16(...args);
}


export function ffi_xcelerate_rust_future_free_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_i16(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i16(...args);
}


export function ffi_xcelerate_rust_future_complete_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i16_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_u32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u32(...args);
}


export function ffi_xcelerate_rust_future_poll_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_u32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u32(...args);
}


export function ffi_xcelerate_rust_future_cancel_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_u32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u32(...args);
}


export function ffi_xcelerate_rust_future_free_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_u32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u32(...args);
}


export function ffi_xcelerate_rust_future_complete_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_i32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i32(...args);
}


export function ffi_xcelerate_rust_future_poll_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_i32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i32(...args);
}


export function ffi_xcelerate_rust_future_cancel_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_i32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i32(...args);
}


export function ffi_xcelerate_rust_future_free_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_i32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i32(...args);
}


export function ffi_xcelerate_rust_future_complete_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_u64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u64(...args);
}


export function ffi_xcelerate_rust_future_poll_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_u64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_u64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(...args);
}


export function ffi_xcelerate_rust_future_cancel_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_u64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_u64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u64(...args);
}


export function ffi_xcelerate_rust_future_free_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_u64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_u64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u64(...args);
}


export function ffi_xcelerate_rust_future_complete_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_u64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_i64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i64(...args);
}


export function ffi_xcelerate_rust_future_poll_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_i64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_i64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i64(...args);
}


export function ffi_xcelerate_rust_future_cancel_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_i64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_i64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i64(...args);
}


export function ffi_xcelerate_rust_future_free_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_i64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_i64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i64(...args);
}


export function ffi_xcelerate_rust_future_complete_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_i64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_f32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_f32(...args);
}


export function ffi_xcelerate_rust_future_poll_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_f32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_f32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_f32(...args);
}


export function ffi_xcelerate_rust_future_cancel_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_f32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_f32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_f32(...args);
}


export function ffi_xcelerate_rust_future_free_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_f32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_f32(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_f32(...args);
}


export function ffi_xcelerate_rust_future_complete_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_f32_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_f64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_f64(...args);
}


export function ffi_xcelerate_rust_future_poll_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_f64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_f64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_f64(...args);
}


export function ffi_xcelerate_rust_future_cancel_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_f64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_f64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_f64(...args);
}


export function ffi_xcelerate_rust_future_free_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_f64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_f64(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_f64(...args);
}


export function ffi_xcelerate_rust_future_complete_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_f64_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(...args);
}


export function ffi_xcelerate_rust_future_poll_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(...args);
}


export function ffi_xcelerate_rust_future_cancel_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(...args);
}


export function ffi_xcelerate_rust_future_free_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(...args);
}


export function ffi_xcelerate_rust_future_complete_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_poll_void(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_void(...args);
}


export function ffi_xcelerate_rust_future_poll_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_poll_void_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_cancel_void(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_void(...args);
}


export function ffi_xcelerate_rust_future_cancel_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_cancel_void_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_free_void(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_void(...args);
}


export function ffi_xcelerate_rust_future_free_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_free_void_generic_abi(...args);
}



export function ffi_xcelerate_rust_future_complete_void(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_void(...args);
}


export function ffi_xcelerate_rust_future_complete_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_rust_future_complete_void_generic_abi(...args);
}



export function uniffi_xcelerate_checksum_method_browser_audit_log(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_audit_log(...args);
}



export function uniffi_xcelerate_checksum_method_browser_audit_verify(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_audit_verify(...args);
}



export function uniffi_xcelerate_checksum_method_browser_available_plugins(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_available_plugins(...args);
}



export function uniffi_xcelerate_checksum_method_browser_browser_contexts(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_browser_contexts(...args);
}



export function uniffi_xcelerate_checksum_method_browser_capabilities(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_capabilities(...args);
}



export function uniffi_xcelerate_checksum_method_browser_close(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_close(...args);
}



export function uniffi_xcelerate_checksum_method_browser_cookies(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_cookies(...args);
}



export function uniffi_xcelerate_checksum_method_browser_delete_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_delete_cookie(...args);
}



export function uniffi_xcelerate_checksum_method_browser_event_names(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_event_names(...args);
}



export function uniffi_xcelerate_checksum_method_browser_grant_permissions(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_grant_permissions(...args);
}



export function uniffi_xcelerate_checksum_method_browser_is_connected(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_is_connected(...args);
}



export function uniffi_xcelerate_checksum_method_browser_listens_to(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_listens_to(...args);
}



export function uniffi_xcelerate_checksum_method_browser_load_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_load_plugin(...args);
}



export function uniffi_xcelerate_checksum_method_browser_new_context(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_new_context(...args);
}



export function uniffi_xcelerate_checksum_method_browser_new_page(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_new_page(...args);
}



export function uniffi_xcelerate_checksum_method_browser_on(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_on(...args);
}



export function uniffi_xcelerate_checksum_method_browser_once(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_once(...args);
}



export function uniffi_xcelerate_checksum_method_browser_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_plugin(...args);
}



export function uniffi_xcelerate_checksum_method_browser_plugin_names(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_plugin_names(...args);
}



export function uniffi_xcelerate_checksum_method_browser_remove_all_listeners(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_remove_all_listeners(...args);
}



export function uniffi_xcelerate_checksum_method_browser_remove_listener(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_remove_listener(...args);
}



export function uniffi_xcelerate_checksum_method_browser_reset_permissions(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_reset_permissions(...args);
}



export function uniffi_xcelerate_checksum_method_browser_set_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_set_cookie(...args);
}



export function uniffi_xcelerate_checksum_method_browser_set_download_behavior(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_set_download_behavior(...args);
}



export function uniffi_xcelerate_checksum_method_browser_start_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_start_tracing(...args);
}



export function uniffi_xcelerate_checksum_method_browser_stop_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_stop_tracing(...args);
}



export function uniffi_xcelerate_checksum_method_browser_targets(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_targets(...args);
}



export function uniffi_xcelerate_checksum_method_browser_use_plugin(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_use_plugin(...args);
}



export function uniffi_xcelerate_checksum_method_browser_user_agent(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_user_agent(...args);
}



export function uniffi_xcelerate_checksum_method_browser_version(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_version(...args);
}



export function uniffi_xcelerate_checksum_method_browser_wait_for_event(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_wait_for_event(...args);
}



export function uniffi_xcelerate_checksum_method_browser_wait_for_event_default(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_wait_for_event_default(...args);
}



export function uniffi_xcelerate_checksum_method_browser_ws_endpoint(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_browser_ws_endpoint(...args);
}



export function uniffi_xcelerate_checksum_method_element_attribute(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_attribute(...args);
}



export function uniffi_xcelerate_checksum_method_element_call_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_call_bool(...args);
}



export function uniffi_xcelerate_checksum_method_element_call_json(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_call_json(...args);
}



export function uniffi_xcelerate_checksum_method_element_call_on_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_call_on_selector(...args);
}



export function uniffi_xcelerate_checksum_method_element_call_on_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_call_on_selector_all(...args);
}



export function uniffi_xcelerate_checksum_method_element_call_string(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_call_string(...args);
}



export function uniffi_xcelerate_checksum_method_element_click(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_click(...args);
}



export function uniffi_xcelerate_checksum_method_element_click_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_click_mouse(...args);
}



export function uniffi_xcelerate_checksum_method_element_count(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_count(...args);
}



export function uniffi_xcelerate_checksum_method_element_dispose(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_dispose(...args);
}



export function uniffi_xcelerate_checksum_method_element_evaluate_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_bool(...args);
}



export function uniffi_xcelerate_checksum_method_element_evaluate_handle(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_handle(...args);
}



export function uniffi_xcelerate_checksum_method_element_evaluate_json(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_json(...args);
}



export function uniffi_xcelerate_checksum_method_element_evaluate_string(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_evaluate_string(...args);
}



export function uniffi_xcelerate_checksum_method_element_focus(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_focus(...args);
}



export function uniffi_xcelerate_checksum_method_element_get_by_label(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_label(...args);
}



export function uniffi_xcelerate_checksum_method_element_get_by_role(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_role(...args);
}



export function uniffi_xcelerate_checksum_method_element_get_by_text(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_get_by_text(...args);
}



export function uniffi_xcelerate_checksum_method_element_get_properties(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_get_properties(...args);
}



export function uniffi_xcelerate_checksum_method_element_hover(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_hover(...args);
}



export function uniffi_xcelerate_checksum_method_element_hover_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_hover_mouse(...args);
}



export function uniffi_xcelerate_checksum_method_element_inner_html(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_inner_html(...args);
}



export function uniffi_xcelerate_checksum_method_element_press(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_press(...args);
}



export function uniffi_xcelerate_checksum_method_element_query_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector(...args);
}



export function uniffi_xcelerate_checksum_method_element_query_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_all(...args);
}



export function uniffi_xcelerate_checksum_method_element_query_selector_attr(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_attr(...args);
}



export function uniffi_xcelerate_checksum_method_element_query_selector_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_query_selector_xpath(...args);
}



export function uniffi_xcelerate_checksum_method_element_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_screenshot(...args);
}



export function uniffi_xcelerate_checksum_method_element_screenshot_base64(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_screenshot_base64(...args);
}



export function uniffi_xcelerate_checksum_method_element_select_option(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_select_option(...args);
}



export function uniffi_xcelerate_checksum_method_element_set_input_files(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_set_input_files(...args);
}



export function uniffi_xcelerate_checksum_method_element_text(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_text(...args);
}



export function uniffi_xcelerate_checksum_method_element_type_text(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_type_text(...args);
}



export function uniffi_xcelerate_checksum_method_element_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_element_wait_for_selector(...args);
}



export function uniffi_xcelerate_checksum_method_page_activate(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_activate(...args);
}



export function uniffi_xcelerate_checksum_method_page_activate_target(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_activate_target(...args);
}



export function uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document(...args);
}



export function uniffi_xcelerate_checksum_method_page_add_style_tag(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_add_style_tag(...args);
}



export function uniffi_xcelerate_checksum_method_page_authenticate(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_authenticate(...args);
}



export function uniffi_xcelerate_checksum_method_page_bring_to_front(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_bring_to_front(...args);
}



export function uniffi_xcelerate_checksum_method_page_call_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_call_bool(...args);
}



export function uniffi_xcelerate_checksum_method_page_call_json(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_call_json(...args);
}



export function uniffi_xcelerate_checksum_method_page_call_on_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_call_on_selector(...args);
}



export function uniffi_xcelerate_checksum_method_page_call_on_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_call_on_selector_all(...args);
}



export function uniffi_xcelerate_checksum_method_page_call_string(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_call_string(...args);
}



export function uniffi_xcelerate_checksum_method_page_clear_requests(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_clear_requests(...args);
}



export function uniffi_xcelerate_checksum_method_page_click_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_click_mouse(...args);
}



export function uniffi_xcelerate_checksum_method_page_close(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_close(...args);
}



export function uniffi_xcelerate_checksum_method_page_content(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_content(...args);
}



export function uniffi_xcelerate_checksum_method_page_cookie(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_cookie(...args);
}



export function uniffi_xcelerate_checksum_method_page_cookies(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_cookies(...args);
}



export function uniffi_xcelerate_checksum_method_page_coverage_start_css(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_start_css(...args);
}



export function uniffi_xcelerate_checksum_method_page_coverage_start_js(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_start_js(...args);
}



export function uniffi_xcelerate_checksum_method_page_coverage_stop_css(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_stop_css(...args);
}



export function uniffi_xcelerate_checksum_method_page_coverage_stop_js(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_coverage_stop_js(...args);
}



export function uniffi_xcelerate_checksum_method_page_create_pdf_stream(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_create_pdf_stream(...args);
}



export function uniffi_xcelerate_checksum_method_page_decode_base64(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_decode_base64(...args);
}



export function uniffi_xcelerate_checksum_method_page_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_default_timeout(...args);
}



export function uniffi_xcelerate_checksum_method_page_document_element(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_document_element(...args);
}



export function uniffi_xcelerate_checksum_method_page_emulate_idle_state(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_emulate_idle_state(...args);
}



export function uniffi_xcelerate_checksum_method_page_emulate_media(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_emulate_media(...args);
}



export function uniffi_xcelerate_checksum_method_page_ensure_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_ensure_interception(...args);
}



export function uniffi_xcelerate_checksum_method_page_evaluate_bool(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_bool(...args);
}



export function uniffi_xcelerate_checksum_method_page_evaluate_handle(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_handle(...args);
}



export function uniffi_xcelerate_checksum_method_page_evaluate_json(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_json(...args);
}



export function uniffi_xcelerate_checksum_method_page_evaluate_string(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_evaluate_string(...args);
}



export function uniffi_xcelerate_checksum_method_page_event_names(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_event_names(...args);
}



export function uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_execute_cdp_cmd(...args);
}



export function uniffi_xcelerate_checksum_method_page_find_element(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_find_element(...args);
}



export function uniffi_xcelerate_checksum_method_page_frame(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_frame(...args);
}



export function uniffi_xcelerate_checksum_method_page_frame_name(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_frame_name(...args);
}



export function uniffi_xcelerate_checksum_method_page_frames(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_frames(...args);
}



export function uniffi_xcelerate_checksum_method_page_get_by_label(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_label(...args);
}



export function uniffi_xcelerate_checksum_method_page_get_by_role(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_role(...args);
}



export function uniffi_xcelerate_checksum_method_page_get_by_text(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_get_by_text(...args);
}



export function uniffi_xcelerate_checksum_method_page_get_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_get_default_timeout(...args);
}



export function uniffi_xcelerate_checksum_method_page_go_back(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_go_back(...args);
}



export function uniffi_xcelerate_checksum_method_page_go_forward(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_go_forward(...args);
}



export function uniffi_xcelerate_checksum_method_page_handle_js_dialog(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_handle_js_dialog(...args);
}



export function uniffi_xcelerate_checksum_method_page_inject_file(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_inject_file(...args);
}



export function uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled(...args);
}



export function uniffi_xcelerate_checksum_method_page_keyboard_down(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_down(...args);
}



export function uniffi_xcelerate_checksum_method_page_keyboard_press(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_press(...args);
}



export function uniffi_xcelerate_checksum_method_page_keyboard_type(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_type(...args);
}



export function uniffi_xcelerate_checksum_method_page_keyboard_up(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_keyboard_up(...args);
}



export function uniffi_xcelerate_checksum_method_page_listens_to(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_listens_to(...args);
}



export function uniffi_xcelerate_checksum_method_page_main_frame(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_main_frame(...args);
}



export function uniffi_xcelerate_checksum_method_page_metrics(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_metrics(...args);
}



export function uniffi_xcelerate_checksum_method_page_mouse_down(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_mouse_down(...args);
}



export function uniffi_xcelerate_checksum_method_page_mouse_up(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_mouse_up(...args);
}



export function uniffi_xcelerate_checksum_method_page_move_mouse(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_move_mouse(...args);
}



export function uniffi_xcelerate_checksum_method_page_navigate(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_navigate(...args);
}



export function uniffi_xcelerate_checksum_method_page_on(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_on(...args);
}



export function uniffi_xcelerate_checksum_method_page_once(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_once(...args);
}



export function uniffi_xcelerate_checksum_method_page_pdf(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_pdf(...args);
}



export function uniffi_xcelerate_checksum_method_page_press(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_press(...args);
}



export function uniffi_xcelerate_checksum_method_page_query_selector_all(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_query_selector_all(...args);
}



export function uniffi_xcelerate_checksum_method_page_query_selector_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_query_selector_xpath(...args);
}



export function uniffi_xcelerate_checksum_method_page_raw_window_bounds(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_raw_window_bounds(...args);
}



export function uniffi_xcelerate_checksum_method_page_reload(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_reload(...args);
}



export function uniffi_xcelerate_checksum_method_page_remove_all_listeners(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_all_listeners(...args);
}



export function uniffi_xcelerate_checksum_method_page_remove_listener(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_listener(...args);
}



export function uniffi_xcelerate_checksum_method_page_remove_script(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_remove_script(...args);
}



export function uniffi_xcelerate_checksum_method_page_request(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_request(...args);
}



export function uniffi_xcelerate_checksum_method_page_requests(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_requests(...args);
}



export function uniffi_xcelerate_checksum_method_page_route(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_route(...args);
}



export function uniffi_xcelerate_checksum_method_page_route_abort(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_route_abort(...args);
}



export function uniffi_xcelerate_checksum_method_page_route_from_har(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_route_from_har(...args);
}



export function uniffi_xcelerate_checksum_method_page_route_fulfill(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_route_fulfill(...args);
}



export function uniffi_xcelerate_checksum_method_page_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_screenshot(...args);
}



export function uniffi_xcelerate_checksum_method_page_screenshot_full(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_screenshot_full(...args);
}



export function uniffi_xcelerate_checksum_method_page_select_option(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_select_option(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_cache_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_cache_enabled(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_content(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_content(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_default_timeout(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_default_timeout(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_drag_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_drag_interception(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_emulated_media_features(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_emulated_media_features(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_extra_http_headers(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_extra_http_headers(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_input_files(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_input_files(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_javascript_enabled(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_javascript_enabled(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_offline(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_offline(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_request_interception(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_request_interception(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_storage_state(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_storage_state(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_user_agent(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_user_agent(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_viewport_size(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_viewport_size(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_window_bounds(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_bounds(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_window_position(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_position(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_window_size(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_size(...args);
}



export function uniffi_xcelerate_checksum_method_page_set_window_state(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_set_window_state(...args);
}



export function uniffi_xcelerate_checksum_method_page_start_screencast(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_start_screencast(...args);
}



export function uniffi_xcelerate_checksum_method_page_start_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_start_tracing(...args);
}



export function uniffi_xcelerate_checksum_method_page_stop_screencast(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_stop_screencast(...args);
}



export function uniffi_xcelerate_checksum_method_page_stop_tracing(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_stop_tracing(...args);
}



export function uniffi_xcelerate_checksum_method_page_storage_state(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_storage_state(...args);
}



export function uniffi_xcelerate_checksum_method_page_target_id(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_target_id(...args);
}



export function uniffi_xcelerate_checksum_method_page_title(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_title(...args);
}



export function uniffi_xcelerate_checksum_method_page_touch_tap(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_touch_tap(...args);
}



export function uniffi_xcelerate_checksum_method_page_unroute(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_unroute(...args);
}



export function uniffi_xcelerate_checksum_method_page_unroute_all(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_unroute_all(...args);
}



export function uniffi_xcelerate_checksum_method_page_url(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_url(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_event(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_event(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_event_default(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_event_default(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_function(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_function(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_navigation(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_navigation(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_selector(...args);
}



export function uniffi_xcelerate_checksum_method_page_wait_for_xpath(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_wait_for_xpath(...args);
}



export function uniffi_xcelerate_checksum_method_page_window_id(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_window_id(...args);
}



export function uniffi_xcelerate_checksum_method_page_window_position(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_window_position(...args);
}



export function uniffi_xcelerate_checksum_method_page_window_rect(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_window_rect(...args);
}



export function uniffi_xcelerate_checksum_method_page_window_size(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_page_window_size(...args);
}



export function uniffi_xcelerate_checksum_method_pluginhandle_invoke(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_invoke(...args);
}



export function uniffi_xcelerate_checksum_method_pluginhandle_ops(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_ops(...args);
}



export function uniffi_xcelerate_checksum_method_pluginhandle_plugin_name(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_method_pluginhandle_plugin_name(...args);
}



export function uniffi_xcelerate_checksum_constructor_browser_launch(...args) {
  return ffiFunctions.uniffi_xcelerate_checksum_constructor_browser_launch(...args);
}



export function ffi_xcelerate_uniffi_contract_version(...args) {
  return ffiFunctions.ffi_xcelerate_uniffi_contract_version(...args);
}


