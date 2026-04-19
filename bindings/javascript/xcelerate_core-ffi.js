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
  namespace: "xcelerate_core",
  cdylibName: "xcelerate_core",
  stagedLibraryPackageRelativePath: "xcelerate_core.dll",
  bundledPrebuilds: false,
  manualLoad: false,
});

export const ffiIntegrity = Object.freeze({
  contractVersionFunction: "ffi_xcelerate_core_uniffi_contract_version",
  expectedContractVersion: 30,
  checksums: Object.freeze({

    "uniffi_xcelerate_core_checksum_method_browser_close": 16019,

    "uniffi_xcelerate_core_checksum_method_browser_new_page": 19279,

    "uniffi_xcelerate_core_checksum_method_browser_version": 17230,

    "uniffi_xcelerate_core_checksum_method_element_attribute": 9708,

    "uniffi_xcelerate_core_checksum_method_element_click": 6241,

    "uniffi_xcelerate_core_checksum_method_element_focus": 9893,

    "uniffi_xcelerate_core_checksum_method_element_hover": 16363,

    "uniffi_xcelerate_core_checksum_method_element_inner_html": 24695,

    "uniffi_xcelerate_core_checksum_method_element_text": 31459,

    "uniffi_xcelerate_core_checksum_method_element_type_text": 8586,

    "uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document": 4984,

    "uniffi_xcelerate_core_checksum_method_page_content": 49602,

    "uniffi_xcelerate_core_checksum_method_page_find_element": 16421,

    "uniffi_xcelerate_core_checksum_method_page_go_back": 62017,

    "uniffi_xcelerate_core_checksum_method_page_navigate": 12821,

    "uniffi_xcelerate_core_checksum_method_page_pdf": 53529,

    "uniffi_xcelerate_core_checksum_method_page_reload": 61610,

    "uniffi_xcelerate_core_checksum_method_page_screenshot": 26126,

    "uniffi_xcelerate_core_checksum_method_page_screenshot_full": 22357,

    "uniffi_xcelerate_core_checksum_method_page_title": 3520,

    "uniffi_xcelerate_core_checksum_method_page_wait_for_navigation": 46856,

    "uniffi_xcelerate_core_checksum_method_page_wait_for_selector": 32739,

    "uniffi_xcelerate_core_checksum_constructor_browser_launch": 10488,

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

    uniffi_xcelerate_core_fn_clone_browser: library.func("uniffi_xcelerate_core_fn_clone_browser", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_clone_browser_generic_abi: library.func("uniffi_xcelerate_core_fn_clone_browser", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_free_browser: library.func("uniffi_xcelerate_core_fn_free_browser", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_free_browser_generic_abi: library.func("uniffi_xcelerate_core_fn_free_browser", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_constructor_browser_launch: library.func("uniffi_xcelerate_core_fn_constructor_browser_launch", ffiTypes.UniffiHandle, [ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_browser_close: library.func("uniffi_xcelerate_core_fn_method_browser_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_browser_close_generic_abi: library.func("uniffi_xcelerate_core_fn_method_browser_close", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_browser_new_page: library.func("uniffi_xcelerate_core_fn_method_browser_new_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi: library.func("uniffi_xcelerate_core_fn_method_browser_new_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_browser_version: library.func("uniffi_xcelerate_core_fn_method_browser_version", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_browser_version_generic_abi: library.func("uniffi_xcelerate_core_fn_method_browser_version", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_clone_element: library.func("uniffi_xcelerate_core_fn_clone_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_clone_element_generic_abi: library.func("uniffi_xcelerate_core_fn_clone_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_free_element: library.func("uniffi_xcelerate_core_fn_free_element", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_free_element_generic_abi: library.func("uniffi_xcelerate_core_fn_free_element", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_method_element_attribute: library.func("uniffi_xcelerate_core_fn_method_element_attribute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_element_attribute_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_attribute", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_element_click: library.func("uniffi_xcelerate_core_fn_method_element_click", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_element_click_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_click", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_element_focus: library.func("uniffi_xcelerate_core_fn_method_element_focus", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_element_focus_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_focus", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_element_hover: library.func("uniffi_xcelerate_core_fn_method_element_hover", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_element_hover_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_hover", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_element_inner_html: library.func("uniffi_xcelerate_core_fn_method_element_inner_html", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_inner_html", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_element_text: library.func("uniffi_xcelerate_core_fn_method_element_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_element_text_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_element_type_text: library.func("uniffi_xcelerate_core_fn_method_element_type_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_element_type_text_generic_abi: library.func("uniffi_xcelerate_core_fn_method_element_type_text", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_clone_page: library.func("uniffi_xcelerate_core_fn_clone_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_clone_page_generic_abi: library.func("uniffi_xcelerate_core_fn_clone_page", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_free_page: library.func("uniffi_xcelerate_core_fn_free_page", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    uniffi_xcelerate_core_fn_free_page_generic_abi: library.func("uniffi_xcelerate_core_fn_free_page", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document: library.func("uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_page_content: library.func("uniffi_xcelerate_core_fn_method_page_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_content_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_content", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_find_element: library.func("uniffi_xcelerate_core_fn_method_page_find_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_page_find_element_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_find_element", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_page_go_back: library.func("uniffi_xcelerate_core_fn_method_page_go_back", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_go_back_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_go_back", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_navigate: library.func("uniffi_xcelerate_core_fn_method_page_navigate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_page_navigate_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_navigate", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    uniffi_xcelerate_core_fn_method_page_pdf: library.func("uniffi_xcelerate_core_fn_method_page_pdf", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_pdf_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_pdf", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_reload: library.func("uniffi_xcelerate_core_fn_method_page_reload", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_reload_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_reload", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_screenshot: library.func("uniffi_xcelerate_core_fn_method_page_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_screenshot", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_screenshot_full: library.func("uniffi_xcelerate_core_fn_method_page_screenshot_full", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_screenshot_full", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_title: library.func("uniffi_xcelerate_core_fn_method_page_title", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_title_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_title", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_wait_for_navigation: library.func("uniffi_xcelerate_core_fn_method_page_wait_for_navigation", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),

    uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_wait_for_navigation", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle]),


    uniffi_xcelerate_core_fn_method_page_wait_for_selector: library.func("uniffi_xcelerate_core_fn_method_page_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),

    uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi: library.func("uniffi_xcelerate_core_fn_method_page_wait_for_selector", ffiTypes.UniffiHandle, [ffiTypes.UniffiHandle, ffiTypes.RustBuffer]),


    ffi_xcelerate_core_rustbuffer_alloc: library.func("ffi_xcelerate_core_rustbuffer_alloc", ffiTypes.RustBuffer, ["uint64_t", koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rustbuffer_from_bytes: library.func("ffi_xcelerate_core_rustbuffer_from_bytes", ffiTypes.RustBuffer, [ffiTypes.ForeignBytes, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rustbuffer_free: library.func("ffi_xcelerate_core_rustbuffer_free", "void", [ffiTypes.RustBuffer, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rustbuffer_reserve: library.func("ffi_xcelerate_core_rustbuffer_reserve", ffiTypes.RustBuffer, [ffiTypes.RustBuffer, "uint64_t", koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_u8: library.func("ffi_xcelerate_core_rust_future_poll_u8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_u8_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_u8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_u8: library.func("ffi_xcelerate_core_rust_future_cancel_u8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_u8_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_u8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_u8: library.func("ffi_xcelerate_core_rust_future_free_u8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_u8_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_u8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_u8: library.func("ffi_xcelerate_core_rust_future_complete_u8", "uint8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_u8_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_u8", "uint8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_i8: library.func("ffi_xcelerate_core_rust_future_poll_i8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_i8_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_i8", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_i8: library.func("ffi_xcelerate_core_rust_future_cancel_i8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_i8_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_i8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_i8: library.func("ffi_xcelerate_core_rust_future_free_i8", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_i8_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_i8", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_i8: library.func("ffi_xcelerate_core_rust_future_complete_i8", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_i8_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_i8", "int8_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_u16: library.func("ffi_xcelerate_core_rust_future_poll_u16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_u16_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_u16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_u16: library.func("ffi_xcelerate_core_rust_future_cancel_u16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_u16_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_u16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_u16: library.func("ffi_xcelerate_core_rust_future_free_u16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_u16_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_u16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_u16: library.func("ffi_xcelerate_core_rust_future_complete_u16", "uint16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_u16_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_u16", "uint16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_i16: library.func("ffi_xcelerate_core_rust_future_poll_i16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_i16_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_i16", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_i16: library.func("ffi_xcelerate_core_rust_future_cancel_i16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_i16_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_i16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_i16: library.func("ffi_xcelerate_core_rust_future_free_i16", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_i16_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_i16", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_i16: library.func("ffi_xcelerate_core_rust_future_complete_i16", "int16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_i16_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_i16", "int16_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_u32: library.func("ffi_xcelerate_core_rust_future_poll_u32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_u32_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_u32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_u32: library.func("ffi_xcelerate_core_rust_future_cancel_u32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_u32_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_u32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_u32: library.func("ffi_xcelerate_core_rust_future_free_u32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_u32_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_u32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_u32: library.func("ffi_xcelerate_core_rust_future_complete_u32", "uint32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_u32_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_u32", "uint32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_i32: library.func("ffi_xcelerate_core_rust_future_poll_i32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_i32_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_i32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_i32: library.func("ffi_xcelerate_core_rust_future_cancel_i32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_i32_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_i32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_i32: library.func("ffi_xcelerate_core_rust_future_free_i32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_i32_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_i32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_i32: library.func("ffi_xcelerate_core_rust_future_complete_i32", "int32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_i32_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_i32", "int32_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_u64: library.func("ffi_xcelerate_core_rust_future_poll_u64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_u64_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_u64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_u64: library.func("ffi_xcelerate_core_rust_future_cancel_u64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_u64_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_u64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_u64: library.func("ffi_xcelerate_core_rust_future_free_u64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_u64_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_u64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_u64: library.func("ffi_xcelerate_core_rust_future_complete_u64", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_u64_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_u64", "uint64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_i64: library.func("ffi_xcelerate_core_rust_future_poll_i64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_i64_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_i64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_i64: library.func("ffi_xcelerate_core_rust_future_cancel_i64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_i64_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_i64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_i64: library.func("ffi_xcelerate_core_rust_future_free_i64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_i64_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_i64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_i64: library.func("ffi_xcelerate_core_rust_future_complete_i64", "int64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_i64_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_i64", "int64_t", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_f32: library.func("ffi_xcelerate_core_rust_future_poll_f32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_f32_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_f32", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_f32: library.func("ffi_xcelerate_core_rust_future_cancel_f32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_f32_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_f32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_f32: library.func("ffi_xcelerate_core_rust_future_free_f32", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_f32_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_f32", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_f32: library.func("ffi_xcelerate_core_rust_future_complete_f32", "float", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_f32_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_f32", "float", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_f64: library.func("ffi_xcelerate_core_rust_future_poll_f64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_f64_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_f64", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_f64: library.func("ffi_xcelerate_core_rust_future_cancel_f64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_f64_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_f64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_f64: library.func("ffi_xcelerate_core_rust_future_free_f64", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_f64_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_f64", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_f64: library.func("ffi_xcelerate_core_rust_future_complete_f64", "double", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_f64_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_f64", "double", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_rust_buffer: library.func("ffi_xcelerate_core_rust_future_poll_rust_buffer", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_rust_buffer_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_rust_buffer", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_rust_buffer: library.func("ffi_xcelerate_core_rust_future_cancel_rust_buffer", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_rust_buffer_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_rust_buffer", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_rust_buffer: library.func("ffi_xcelerate_core_rust_future_free_rust_buffer", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_rust_buffer_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_rust_buffer", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_rust_buffer: library.func("ffi_xcelerate_core_rust_future_complete_rust_buffer", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_rust_buffer_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_rust_buffer", ffiTypes.RustBuffer, [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    ffi_xcelerate_core_rust_future_poll_void: library.func("ffi_xcelerate_core_rust_future_poll_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_poll_void_generic_abi: library.func("ffi_xcelerate_core_rust_future_poll_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiCallbacks.RustFutureContinuationCallback), ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_cancel_void: library.func("ffi_xcelerate_core_rust_future_cancel_void", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_cancel_void_generic_abi: library.func("ffi_xcelerate_core_rust_future_cancel_void", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_free_void: library.func("ffi_xcelerate_core_rust_future_free_void", "void", [ffiTypes.UniffiHandle]),

    ffi_xcelerate_core_rust_future_free_void_generic_abi: library.func("ffi_xcelerate_core_rust_future_free_void", "void", [ffiTypes.UniffiHandle]),


    ffi_xcelerate_core_rust_future_complete_void: library.func("ffi_xcelerate_core_rust_future_complete_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),

    ffi_xcelerate_core_rust_future_complete_void_generic_abi: library.func("ffi_xcelerate_core_rust_future_complete_void", "void", [ffiTypes.UniffiHandle, koffi.pointer(ffiTypes.RustCallStatus)]),


    uniffi_xcelerate_core_checksum_method_browser_close: library.func("uniffi_xcelerate_core_checksum_method_browser_close", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_browser_new_page: library.func("uniffi_xcelerate_core_checksum_method_browser_new_page", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_browser_version: library.func("uniffi_xcelerate_core_checksum_method_browser_version", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_attribute: library.func("uniffi_xcelerate_core_checksum_method_element_attribute", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_click: library.func("uniffi_xcelerate_core_checksum_method_element_click", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_focus: library.func("uniffi_xcelerate_core_checksum_method_element_focus", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_hover: library.func("uniffi_xcelerate_core_checksum_method_element_hover", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_inner_html: library.func("uniffi_xcelerate_core_checksum_method_element_inner_html", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_text: library.func("uniffi_xcelerate_core_checksum_method_element_text", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_element_type_text: library.func("uniffi_xcelerate_core_checksum_method_element_type_text", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document: library.func("uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_content: library.func("uniffi_xcelerate_core_checksum_method_page_content", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_find_element: library.func("uniffi_xcelerate_core_checksum_method_page_find_element", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_go_back: library.func("uniffi_xcelerate_core_checksum_method_page_go_back", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_navigate: library.func("uniffi_xcelerate_core_checksum_method_page_navigate", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_pdf: library.func("uniffi_xcelerate_core_checksum_method_page_pdf", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_reload: library.func("uniffi_xcelerate_core_checksum_method_page_reload", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_screenshot: library.func("uniffi_xcelerate_core_checksum_method_page_screenshot", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_screenshot_full: library.func("uniffi_xcelerate_core_checksum_method_page_screenshot_full", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_title: library.func("uniffi_xcelerate_core_checksum_method_page_title", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_wait_for_navigation: library.func("uniffi_xcelerate_core_checksum_method_page_wait_for_navigation", "uint16_t", []),


    uniffi_xcelerate_core_checksum_method_page_wait_for_selector: library.func("uniffi_xcelerate_core_checksum_method_page_wait_for_selector", "uint16_t", []),


    uniffi_xcelerate_core_checksum_constructor_browser_launch: library.func("uniffi_xcelerate_core_checksum_constructor_browser_launch", "uint16_t", []),


    ffi_xcelerate_core_uniffi_contract_version: library.func("ffi_xcelerate_core_uniffi_contract_version", "uint32_t", []),


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
  return bindings.ffiFunctions.ffi_xcelerate_core_uniffi_contract_version();
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

    "uniffi_xcelerate_core_checksum_method_browser_close": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_close(),

    "uniffi_xcelerate_core_checksum_method_browser_new_page": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_new_page(),

    "uniffi_xcelerate_core_checksum_method_browser_version": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_version(),

    "uniffi_xcelerate_core_checksum_method_element_attribute": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_attribute(),

    "uniffi_xcelerate_core_checksum_method_element_click": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_click(),

    "uniffi_xcelerate_core_checksum_method_element_focus": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_focus(),

    "uniffi_xcelerate_core_checksum_method_element_hover": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_hover(),

    "uniffi_xcelerate_core_checksum_method_element_inner_html": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_inner_html(),

    "uniffi_xcelerate_core_checksum_method_element_text": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_text(),

    "uniffi_xcelerate_core_checksum_method_element_type_text": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_element_type_text(),

    "uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(),

    "uniffi_xcelerate_core_checksum_method_page_content": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_content(),

    "uniffi_xcelerate_core_checksum_method_page_find_element": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_find_element(),

    "uniffi_xcelerate_core_checksum_method_page_go_back": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_go_back(),

    "uniffi_xcelerate_core_checksum_method_page_navigate": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_navigate(),

    "uniffi_xcelerate_core_checksum_method_page_pdf": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_pdf(),

    "uniffi_xcelerate_core_checksum_method_page_reload": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_reload(),

    "uniffi_xcelerate_core_checksum_method_page_screenshot": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_screenshot(),

    "uniffi_xcelerate_core_checksum_method_page_screenshot_full": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_screenshot_full(),

    "uniffi_xcelerate_core_checksum_method_page_title": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_title(),

    "uniffi_xcelerate_core_checksum_method_page_wait_for_navigation": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(),

    "uniffi_xcelerate_core_checksum_method_page_wait_for_selector": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_method_page_wait_for_selector(),

    "uniffi_xcelerate_core_checksum_constructor_browser_launch": bindings.ffiFunctions.uniffi_xcelerate_core_checksum_constructor_browser_launch(),

  });
}

export function validateChecksums(bindings = getFfiBindings()) {
  const actualChecksums = getChecksums(bindings);

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_browser_close"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_browser_close"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_browser_close", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_browser_new_page"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_browser_new_page"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_browser_new_page", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_browser_version"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_browser_version"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_browser_version", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_attribute"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_attribute"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_attribute", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_click"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_click"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_click", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_focus"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_focus"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_focus", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_hover"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_hover"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_hover", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_inner_html"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_inner_html"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_inner_html", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_text"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_element_type_text"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_element_type_text"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_element_type_text", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_content"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_content"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_content", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_find_element"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_find_element"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_find_element", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_go_back"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_go_back"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_go_back", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_navigate"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_navigate"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_navigate", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_pdf"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_pdf"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_pdf", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_reload"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_reload"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_reload", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_screenshot"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_screenshot"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_screenshot", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_screenshot_full"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_screenshot_full"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_screenshot_full", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_title"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_title"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_title", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_wait_for_navigation"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_wait_for_navigation"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_wait_for_navigation", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_method_page_wait_for_selector"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_method_page_wait_for_selector"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_method_page_wait_for_selector", expected, actual, {
        details: {
          libraryPath: bindings.libraryPath,
          packageRelativePath: bindings.packageRelativePath,
        },
      });
    }
  }

  {
    const expected = ffiIntegrity.checksums["uniffi_xcelerate_core_checksum_constructor_browser_launch"];
    const actual = actualChecksums["uniffi_xcelerate_core_checksum_constructor_browser_launch"];
    if (actual !== expected) {
      throw new ChecksumMismatchError("uniffi_xcelerate_core_checksum_constructor_browser_launch", expected, actual, {
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

  uniffi_xcelerate_core_fn_clone_browser(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_browser(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_clone_browser_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_browser_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_free_browser(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_browser(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_free_browser_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_browser_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_constructor_browser_launch(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_constructor_browser_launch(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_close(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_close_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_close_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_new_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_new_page(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_version(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_version(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_browser_version_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_browser_version_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_clone_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_element(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_clone_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_element_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_free_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_element(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_free_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_element_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_method_element_attribute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_attribute(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_attribute_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_attribute_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_click(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_click(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_click_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_click_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_focus(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_focus(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_focus_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_focus_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_hover(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_hover(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_hover_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_hover_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_inner_html(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_inner_html(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_type_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_type_text(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_element_type_text_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_element_type_text_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_clone_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_page(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_clone_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_clone_page_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_free_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_page(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_free_page_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_free_page_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_content(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_content_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_content_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_find_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_find_element(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_find_element_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_find_element_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_go_back(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_go_back(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_go_back_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_go_back_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_navigate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_navigate(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_navigate_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_navigate_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_pdf(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_pdf(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_pdf_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_pdf_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_reload(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_reload(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_reload_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_reload_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_screenshot(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_screenshot_full(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_screenshot_full(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_title(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_title(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_title_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_title_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_wait_for_navigation(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_wait_for_navigation(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_wait_for_selector(...args);

    return normalizeHandle(result);

  },


  uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi(...args);

    return normalizeHandle(result);

  },


  ffi_xcelerate_core_rustbuffer_alloc(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rustbuffer_alloc(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_core_rustbuffer_from_bytes(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rustbuffer_from_bytes(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_core_rustbuffer_free(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rustbuffer_free(...args);

    return result;

  },


  ffi_xcelerate_core_rustbuffer_reserve(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rustbuffer_reserve(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_core_rust_future_poll_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i8(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i8(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i8_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i8_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i16(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i16(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i16_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i16_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_u64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_u64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u64(...args);

    return normalizeUInt64(result);

  },


  ffi_xcelerate_core_rust_future_complete_u64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_u64_generic_abi(...args);

    return normalizeUInt64(result);

  },


  ffi_xcelerate_core_rust_future_poll_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_i64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_i64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i64(...args);

    return normalizeInt64(result);

  },


  ffi_xcelerate_core_rust_future_complete_i64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_i64_generic_abi(...args);

    return normalizeInt64(result);

  },


  ffi_xcelerate_core_rust_future_poll_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_f32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_f32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_f32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_f32(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_f32(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_f32_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_f32_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_f64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_f64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_f64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_f64(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_f64(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_f64_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_f64_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_rust_buffer(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_rust_buffer_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_rust_buffer(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_rust_buffer(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_core_rust_future_complete_rust_buffer_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_rust_buffer_generic_abi(...args);

    return normalizeRustBuffer(result);

  },


  ffi_xcelerate_core_rust_future_poll_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_void(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_poll_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_poll_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_void(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_cancel_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_cancel_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_void(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_free_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_free_void_generic_abi(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_void(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_void(...args);

    return result;

  },


  ffi_xcelerate_core_rust_future_complete_void_generic_abi(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_rust_future_complete_void_generic_abi(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_browser_close(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_browser_close(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_browser_new_page(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_browser_new_page(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_browser_version(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_browser_version(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_attribute(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_attribute(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_click(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_click(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_focus(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_focus(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_hover(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_hover(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_inner_html(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_inner_html(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_text(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_element_type_text(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_element_type_text(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_content(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_content(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_find_element(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_find_element(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_go_back(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_go_back(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_navigate(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_navigate(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_pdf(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_pdf(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_reload(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_reload(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_screenshot(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_screenshot(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_screenshot_full(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_screenshot_full(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_title(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_title(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_method_page_wait_for_selector(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_method_page_wait_for_selector(...args);

    return result;

  },


  uniffi_xcelerate_core_checksum_constructor_browser_launch(...args) {
    const result = getLoadedFfiFunctions().uniffi_xcelerate_core_checksum_constructor_browser_launch(...args);

    return result;

  },


  ffi_xcelerate_core_uniffi_contract_version(...args) {
    const result = getLoadedFfiFunctions().ffi_xcelerate_core_uniffi_contract_version(...args);

    return result;

  },


});


export function uniffi_xcelerate_core_fn_clone_browser(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_browser(...args);
}


export function uniffi_xcelerate_core_fn_clone_browser_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_browser_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_free_browser(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_browser(...args);
}


export function uniffi_xcelerate_core_fn_free_browser_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_browser_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_constructor_browser_launch(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_constructor_browser_launch(...args);
}



export function uniffi_xcelerate_core_fn_method_browser_close(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_close(...args);
}


export function uniffi_xcelerate_core_fn_method_browser_close_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_close_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_browser_new_page(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_new_page(...args);
}


export function uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_browser_version(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_version(...args);
}


export function uniffi_xcelerate_core_fn_method_browser_version_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_browser_version_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_clone_element(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_element(...args);
}


export function uniffi_xcelerate_core_fn_clone_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_element_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_free_element(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_element(...args);
}


export function uniffi_xcelerate_core_fn_free_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_element_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_attribute(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_attribute(...args);
}


export function uniffi_xcelerate_core_fn_method_element_attribute_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_attribute_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_click(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_click(...args);
}


export function uniffi_xcelerate_core_fn_method_element_click_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_click_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_focus(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_focus(...args);
}


export function uniffi_xcelerate_core_fn_method_element_focus_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_focus_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_hover(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_hover(...args);
}


export function uniffi_xcelerate_core_fn_method_element_hover_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_hover_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_inner_html(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_inner_html(...args);
}


export function uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_text(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_text(...args);
}


export function uniffi_xcelerate_core_fn_method_element_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_text_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_element_type_text(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_type_text(...args);
}


export function uniffi_xcelerate_core_fn_method_element_type_text_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_element_type_text_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_clone_page(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_page(...args);
}


export function uniffi_xcelerate_core_fn_clone_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_clone_page_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_free_page(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_page(...args);
}


export function uniffi_xcelerate_core_fn_free_page_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_free_page_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document(...args);
}


export function uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_content(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_content(...args);
}


export function uniffi_xcelerate_core_fn_method_page_content_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_content_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_find_element(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_find_element(...args);
}


export function uniffi_xcelerate_core_fn_method_page_find_element_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_find_element_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_go_back(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_go_back(...args);
}


export function uniffi_xcelerate_core_fn_method_page_go_back_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_go_back_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_navigate(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_navigate(...args);
}


export function uniffi_xcelerate_core_fn_method_page_navigate_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_navigate_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_pdf(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_pdf(...args);
}


export function uniffi_xcelerate_core_fn_method_page_pdf_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_pdf_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_reload(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_reload(...args);
}


export function uniffi_xcelerate_core_fn_method_page_reload_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_reload_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot(...args);
}


export function uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_screenshot_full(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_full(...args);
}


export function uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_title(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_title(...args);
}


export function uniffi_xcelerate_core_fn_method_page_title_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_title_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_wait_for_navigation(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_navigation(...args);
}


export function uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi(...args);
}



export function uniffi_xcelerate_core_fn_method_page_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_selector(...args);
}


export function uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi(...args) {
  return ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi(...args);
}



export function ffi_xcelerate_core_rustbuffer_alloc(...args) {
  return ffiFunctions.ffi_xcelerate_core_rustbuffer_alloc(...args);
}



export function ffi_xcelerate_core_rustbuffer_from_bytes(...args) {
  return ffiFunctions.ffi_xcelerate_core_rustbuffer_from_bytes(...args);
}



export function ffi_xcelerate_core_rustbuffer_free(...args) {
  return ffiFunctions.ffi_xcelerate_core_rustbuffer_free(...args);
}



export function ffi_xcelerate_core_rustbuffer_reserve(...args) {
  return ffiFunctions.ffi_xcelerate_core_rustbuffer_reserve(...args);
}



export function ffi_xcelerate_core_rust_future_poll_u8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u8(...args);
}


export function ffi_xcelerate_core_rust_future_poll_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_u8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u8(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_u8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u8(...args);
}


export function ffi_xcelerate_core_rust_future_free_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_u8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u8(...args);
}


export function ffi_xcelerate_core_rust_future_complete_u8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_i8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i8(...args);
}


export function ffi_xcelerate_core_rust_future_poll_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_i8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i8(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_i8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i8(...args);
}


export function ffi_xcelerate_core_rust_future_free_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_i8(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i8(...args);
}


export function ffi_xcelerate_core_rust_future_complete_i8_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i8_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_u16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u16(...args);
}


export function ffi_xcelerate_core_rust_future_poll_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_u16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u16(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_u16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u16(...args);
}


export function ffi_xcelerate_core_rust_future_free_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_u16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u16(...args);
}


export function ffi_xcelerate_core_rust_future_complete_u16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_i16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i16(...args);
}


export function ffi_xcelerate_core_rust_future_poll_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_i16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i16(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_i16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i16(...args);
}


export function ffi_xcelerate_core_rust_future_free_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_i16(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i16(...args);
}


export function ffi_xcelerate_core_rust_future_complete_i16_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i16_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_u32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u32(...args);
}


export function ffi_xcelerate_core_rust_future_poll_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_u32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u32(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_u32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u32(...args);
}


export function ffi_xcelerate_core_rust_future_free_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_u32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u32(...args);
}


export function ffi_xcelerate_core_rust_future_complete_u32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_i32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i32(...args);
}


export function ffi_xcelerate_core_rust_future_poll_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_i32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i32(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_i32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i32(...args);
}


export function ffi_xcelerate_core_rust_future_free_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_i32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i32(...args);
}


export function ffi_xcelerate_core_rust_future_complete_i32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_u64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(...args);
}


export function ffi_xcelerate_core_rust_future_poll_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_u64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_u64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(...args);
}


export function ffi_xcelerate_core_rust_future_free_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_u64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_u64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u64(...args);
}


export function ffi_xcelerate_core_rust_future_complete_u64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_u64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_i64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i64(...args);
}


export function ffi_xcelerate_core_rust_future_poll_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_i64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_i64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i64(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_i64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_i64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i64(...args);
}


export function ffi_xcelerate_core_rust_future_free_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_i64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_i64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i64(...args);
}


export function ffi_xcelerate_core_rust_future_complete_i64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_i64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_f32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_f32(...args);
}


export function ffi_xcelerate_core_rust_future_poll_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_f32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_f32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_f32(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_f32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_f32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_f32(...args);
}


export function ffi_xcelerate_core_rust_future_free_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_f32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_f32(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_f32(...args);
}


export function ffi_xcelerate_core_rust_future_complete_f32_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_f32_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_f64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_f64(...args);
}


export function ffi_xcelerate_core_rust_future_poll_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_f64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_f64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_f64(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_f64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_f64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_f64(...args);
}


export function ffi_xcelerate_core_rust_future_free_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_f64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_f64(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_f64(...args);
}


export function ffi_xcelerate_core_rust_future_complete_f64_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_f64_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(...args);
}


export function ffi_xcelerate_core_rust_future_poll_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(...args);
}


export function ffi_xcelerate_core_rust_future_free_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_rust_buffer(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(...args);
}


export function ffi_xcelerate_core_rust_future_complete_rust_buffer_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_poll_void(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(...args);
}


export function ffi_xcelerate_core_rust_future_poll_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_poll_void_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_cancel_void(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(...args);
}


export function ffi_xcelerate_core_rust_future_cancel_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_free_void(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_void(...args);
}


export function ffi_xcelerate_core_rust_future_free_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_free_void_generic_abi(...args);
}



export function ffi_xcelerate_core_rust_future_complete_void(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(...args);
}


export function ffi_xcelerate_core_rust_future_complete_void_generic_abi(...args) {
  return ffiFunctions.ffi_xcelerate_core_rust_future_complete_void_generic_abi(...args);
}



export function uniffi_xcelerate_core_checksum_method_browser_close(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_close(...args);
}



export function uniffi_xcelerate_core_checksum_method_browser_new_page(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_new_page(...args);
}



export function uniffi_xcelerate_core_checksum_method_browser_version(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_browser_version(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_attribute(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_attribute(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_click(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_click(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_focus(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_focus(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_hover(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_hover(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_inner_html(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_inner_html(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_text(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_text(...args);
}



export function uniffi_xcelerate_core_checksum_method_element_type_text(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_element_type_text(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_add_script_to_evaluate_on_new_document(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_content(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_content(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_find_element(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_find_element(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_go_back(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_go_back(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_navigate(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_navigate(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_pdf(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_pdf(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_reload(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_reload(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_screenshot(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_screenshot(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_screenshot_full(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_screenshot_full(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_title(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_title(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_wait_for_navigation(...args);
}



export function uniffi_xcelerate_core_checksum_method_page_wait_for_selector(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_method_page_wait_for_selector(...args);
}



export function uniffi_xcelerate_core_checksum_constructor_browser_launch(...args) {
  return ffiFunctions.uniffi_xcelerate_core_checksum_constructor_browser_launch(...args);
}



export function ffi_xcelerate_core_uniffi_contract_version(...args) {
  return ffiFunctions.ffi_xcelerate_core_uniffi_contract_version(...args);
}


