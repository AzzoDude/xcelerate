import koffi from "koffi";


import {

  configureRuntimeHooks,

  ffiFunctions,

  getFfiBindings,

  getFfiTypes,

} from "./xcelerate-ffi.js";


import {

  createForeignBytes,

  RustBufferValue,

} from "./runtime/ffi-types.js";


import {

  AbstractFfiConverterByteArray,

  FfiConverterArray,

  FfiConverterBool,

  FfiConverterBytes,

  FfiConverterFloat64,

  FfiConverterInt64,

  FfiConverterOptional,

  FfiConverterString,

  FfiConverterUInt64,

} from "./runtime/ffi-converters.js";


import {

  UnexpectedEnumCase,

} from "./runtime/errors.js";


import {

  rustCallAsync,

  rustFutureContinuationCallback,

} from "./runtime/async-rust-call.js";


import {

  clearPendingForeignFutures,

} from "./runtime/callbacks.js";


import {

  createObjectConverter,

  createObjectFactory,

  UniffiObjectBase,

} from "./runtime/objects.js";


import {

  UniffiRustCaller,

  createRustCallStatus,

} from "./runtime/rust-call.js";


export const componentMetadata = Object.freeze({
  namespace: "xcelerate",
  packageName: "xcelerate",
  cdylibName: "xcelerate",
  nodeEngine: ">=16",
  bundledPrebuilds: false,
  manualLoad: false,
});

export { ffiMetadata } from "./xcelerate-ffi.js";


function uniffiNotImplemented(member) {
  throw new Error(`${member} is not implemented yet. Koffi-backed bindings are still pending.`);
}

const uniffiTextEncoder = new TextEncoder();
const uniffiTextDecoder = new TextDecoder();
const uniffiSuccessRustCallStatus = createRustCallStatus();
const UNIFFI_MAX_CACHED_RUST_CALL_STATUSES = 32;
const uniffiRustCallStatusPool = [];

function uniffiLiftString(bytes) {
  return uniffiTextDecoder.decode(bytes);
}

function uniffiDecodeRustCallStatus(status) {
  return status == null
    ? uniffiSuccessRustCallStatus
    : koffi.decode(status, 0, "int8_t") === 0
      ? uniffiSuccessRustCallStatus
      : koffi.decode(status, getFfiTypes().RustCallStatus);
}

function uniffiWriteRustCallStatus(status, value) {
  if (
    status != null
    && (
      value.code !== 0
      || value.error_buf.data !== null
      || value.error_buf.len !== 0n
      || value.error_buf.capacity !== 0n
    )
  ) {
    koffi.encode(status, getFfiTypes().RustCallStatus, value);
  }
  return status;
}

function uniffiAcquireRustCallStatus() {
  const status = uniffiRustCallStatusPool.pop();
  if (status != null) {
    return status;
  }
  return koffi.alloc(getFfiTypes().RustCallStatus, 1);
}

function uniffiReleaseRustCallStatus(status) {
  if (status == null) {
    return;
  }

  koffi.encode(status, getFfiTypes().RustCallStatus, uniffiSuccessRustCallStatus);
  if (uniffiRustCallStatusPool.length < UNIFFI_MAX_CACHED_RUST_CALL_STATUSES) {
    uniffiRustCallStatusPool.push(status);
    return;
  }

  koffi.free(status);
}

const uniffiRustCaller = new UniffiRustCaller({
  createStatus: uniffiAcquireRustCallStatus,
  disposeStatus: uniffiReleaseRustCallStatus,
  readStatus: uniffiDecodeRustCallStatus,
  writeStatus: uniffiWriteRustCallStatus,
  liftString: uniffiLiftString,
});

function uniffiFreeRustBuffer(buffer) {
  return uniffiRustCaller.rustCall(
    (status) => ffiFunctions.ffi_xcelerate_rustbuffer_free(buffer, status),
    { liftString: uniffiLiftString },
  );
}

const uniffiDefaultRustCallOptions = Object.freeze({
  freeRustBuffer: uniffiFreeRustBuffer,
  liftString: uniffiLiftString,
  rustCaller: uniffiRustCaller,
});
const uniffiRustCallOptionsByErrorConverter = new WeakMap();
const uniffiOptionalConverterCache = new WeakMap();
const uniffiArrayConverterCache = new WeakMap();
const uniffiLibraryFunctionCache = new WeakMap();

function uniffiRustCallOptions(errorConverter = undefined) {
  if (errorConverter == null) {
    return uniffiDefaultRustCallOptions;
  }

  if (typeof errorConverter !== "object" && typeof errorConverter !== "function") {
    return Object.freeze({
      errorHandler: (errorBytes) => errorConverter.lift(errorBytes),
      freeRustBuffer: uniffiFreeRustBuffer,
      liftString: uniffiLiftString,
      rustCaller: uniffiRustCaller,
    });
  }

  const cachedOptions = uniffiRustCallOptionsByErrorConverter.get(errorConverter);
  if (cachedOptions != null) {
    return cachedOptions;
  }

  const options = Object.freeze({
    errorHandler: (errorBytes) => errorConverter.lift(errorBytes),
    freeRustBuffer: uniffiFreeRustBuffer,
    liftString: uniffiLiftString,
    rustCaller: uniffiRustCaller,
  });
  uniffiRustCallOptionsByErrorConverter.set(errorConverter, options);
  return options;
}

function uniffiOptionalConverter(innerConverter) {
  let converter = uniffiOptionalConverterCache.get(innerConverter);
  if (converter == null) {
    converter = new FfiConverterOptional(innerConverter);
    uniffiOptionalConverterCache.set(innerConverter, converter);
  }
  return converter;
}

function uniffiArrayConverter(innerConverter) {
  let converter = uniffiArrayConverterCache.get(innerConverter);
  if (converter == null) {
    converter = new FfiConverterArray(innerConverter);
    uniffiArrayConverterCache.set(innerConverter, converter);
  }
  return converter;
}

function uniffiGetCachedLibraryFunction(cacheKey, create) {
  const bindings = getFfiBindings();
  const library = bindings.library;
  let libraryCache = uniffiLibraryFunctionCache.get(library);
  if (libraryCache == null) {
    libraryCache = new Map();
    uniffiLibraryFunctionCache.set(library, libraryCache);
  }

  let cachedFunction = libraryCache.get(cacheKey);
  if (cachedFunction == null) {
    cachedFunction = create(bindings);
    libraryCache.set(cacheKey, cachedFunction);
  }
  return cachedFunction;
}

function uniffiCopyIntoRustBuffer(bytes) {
  return uniffiRustCaller.rustCall(
    (status) => ffiFunctions.ffi_xcelerate_rustbuffer_from_bytes(createForeignBytes(bytes), status),
    uniffiRustCallOptions(),
  );
}

function uniffiLowerString(value) {
  return uniffiCopyIntoRustBuffer(uniffiTextEncoder.encode(value));
}

function uniffiLiftStringFromRustBuffer(value) {
  return uniffiLiftString(new RustBufferValue(value).consumeIntoUint8Array(uniffiFreeRustBuffer));
}

function uniffiLowerBytes(value) {
  return uniffiCopyIntoRustBuffer(value);
}

function uniffiLiftBytesFromRustBuffer(value) {
  return new RustBufferValue(value).consumeIntoUint8Array(uniffiFreeRustBuffer);
}

function uniffiLowerIntoRustBuffer(converter, value) {
  return uniffiCopyIntoRustBuffer(converter.lower(value));
}

function uniffiLiftFromRustBuffer(converter, value) {
  return converter.lift(uniffiLiftBytesFromRustBuffer(value));
}

function uniffiRequireRecordObject(typeName, value) {
  if (typeof value !== "object" || value == null) {
    throw new TypeError(`${typeName} values must be non-null objects.`);
  }
  return value;
}

function uniffiRequireFlatEnumValue(enumValues, typeName, value) {
  for (const enumValue of Object.values(enumValues)) {
    if (enumValue === value) {
      return enumValue;
    }
  }
  throw new TypeError(`${typeName} values must be one of ${Object.values(enumValues).map((item) => JSON.stringify(item)).join(", ")}.`);
}

function uniffiRequireTaggedEnumValue(typeName, value) {
  const enumValue = uniffiRequireRecordObject(typeName, value);
  if (typeof enumValue.tag !== "string") {
    throw new TypeError(`${typeName} values must be tagged objects with a string tag field.`);
  }
  return enumValue;
}

function uniffiNotImplementedConverter(typeName) {
  const fail = (member) => {
    throw new Error(`${typeName} converter ${member} is not implemented yet.`);
  };
  return Object.freeze({
    lower() {
      return fail("lower");
    },
    lift() {
      return fail("lift");
    },
    write() {
      return fail("write");
    },
    read() {
      return fail("read");
    },
    allocationSize() {
      return fail("allocationSize");
    },
  });
}

let uniffiRustFutureContinuationPointer = null;

function uniffiGetRustFutureContinuationPointer() {
  if (uniffiRustFutureContinuationPointer == null) {
    const bindings = getFfiBindings();
    uniffiRustFutureContinuationPointer = koffi.register(
      rustFutureContinuationCallback,
      koffi.pointer(bindings.ffiCallbacks.RustFutureContinuationCallback),
    );
  }
  return uniffiRustFutureContinuationPointer;
}

export class XcelerateError extends globalThis.Error {
  constructor(tag, message = tag) {
    super(message);
    this.name = "XcelerateError";
    this.tag = tag;
  }
}

export class XcelerateErrorWsError extends XcelerateError {
  constructor(message = undefined) {
    super("WsError", message ?? "WsError");
    this.name = "XcelerateErrorWsError";
    this.message = message ?? "WsError";
  }
}

export class XcelerateErrorSerdeError extends XcelerateError {
  constructor(message = undefined) {
    super("SerdeError", message ?? "SerdeError");
    this.name = "XcelerateErrorSerdeError";
    this.message = message ?? "SerdeError";
  }
}

export class XcelerateErrorCdpResponseError extends XcelerateError {
  constructor(message = undefined) {
    super("CdpResponseError", message ?? "CdpResponseError");
    this.name = "XcelerateErrorCdpResponseError";
    this.message = message ?? "CdpResponseError";
  }
}

export class XcelerateErrorHttpError extends XcelerateError {
  constructor(message = undefined) {
    super("HttpError", message ?? "HttpError");
    this.name = "XcelerateErrorHttpError";
    this.message = message ?? "HttpError";
  }
}

export class XcelerateErrorNotFound extends XcelerateError {
  constructor(message = undefined) {
    super("NotFound", message ?? "NotFound");
    this.name = "XcelerateErrorNotFound";
    this.message = message ?? "NotFound";
  }
}

export class XcelerateErrorInternalError extends XcelerateError {
  constructor(message = undefined) {
    super("InternalError", message ?? "InternalError");
    this.name = "XcelerateErrorInternalError";
    this.message = message ?? "InternalError";
  }
}

export class XcelerateErrorUnsupported extends XcelerateError {
  constructor(message = undefined) {
    super("Unsupported", message ?? "Unsupported");
    this.name = "XcelerateErrorUnsupported";
    this.message = message ?? "Unsupported";
  }
}

const FfiConverterBrowserConfig = new (class extends AbstractFfiConverterByteArray {
  allocationSize(value) {
    const recordValue = uniffiRequireRecordObject("BrowserConfig", value);
    return FfiConverterBool.allocationSize(recordValue["headless"]) + FfiConverterBool.allocationSize(recordValue["stealth"]) + FfiConverterBool.allocationSize(recordValue["detached"]) + uniffiOptionalConverter(FfiConverterString).allocationSize(recordValue["executable_path"]) + uniffiOptionalConverter(uniffiArrayConverter(FfiConverterString)).allocationSize(recordValue["plugins"]);
  }

  write(value, writer) {
    const recordValue = uniffiRequireRecordObject("BrowserConfig", value);
    FfiConverterBool.write(recordValue["headless"], writer);
    FfiConverterBool.write(recordValue["stealth"], writer);
    FfiConverterBool.write(recordValue["detached"], writer);
    uniffiOptionalConverter(FfiConverterString).write(recordValue["executable_path"], writer);
    uniffiOptionalConverter(uniffiArrayConverter(FfiConverterString)).write(recordValue["plugins"], writer);
  }

  read(reader) {
    return {
      "headless": FfiConverterBool.read(reader),
      "stealth": FfiConverterBool.read(reader),
      "detached": FfiConverterBool.read(reader),
      "executable_path": uniffiOptionalConverter(FfiConverterString).read(reader),
      "plugins": uniffiOptionalConverter(uniffiArrayConverter(FfiConverterString)).read(reader),
    };
  }
})();
const FfiConverterXcelerateError = new (class extends AbstractFfiConverterByteArray {
  allocationSize(value) {
    if (value instanceof XcelerateErrorWsError) {
      return 4;
    }
    if (value instanceof XcelerateErrorSerdeError) {
      return 4;
    }
    if (value instanceof XcelerateErrorCdpResponseError) {
      return 4;
    }
    if (value instanceof XcelerateErrorHttpError) {
      return 4;
    }
    if (value instanceof XcelerateErrorNotFound) {
      return 4;
    }
    if (value instanceof XcelerateErrorInternalError) {
      return 4;
    }
    if (value instanceof XcelerateErrorUnsupported) {
      return 4;
    }
    throw new TypeError("XcelerateError values must be instances of XcelerateErrorWsError, XcelerateErrorSerdeError, XcelerateErrorCdpResponseError, XcelerateErrorHttpError, XcelerateErrorNotFound, XcelerateErrorInternalError, XcelerateErrorUnsupported.");
  }

  write(value, writer) {
    if (value instanceof XcelerateErrorWsError) {
      writer.writeInt32(1);
      return;
    }
    if (value instanceof XcelerateErrorSerdeError) {
      writer.writeInt32(2);
      return;
    }
    if (value instanceof XcelerateErrorCdpResponseError) {
      writer.writeInt32(3);
      return;
    }
    if (value instanceof XcelerateErrorHttpError) {
      writer.writeInt32(4);
      return;
    }
    if (value instanceof XcelerateErrorNotFound) {
      writer.writeInt32(5);
      return;
    }
    if (value instanceof XcelerateErrorInternalError) {
      writer.writeInt32(6);
      return;
    }
    if (value instanceof XcelerateErrorUnsupported) {
      writer.writeInt32(7);
      return;
    }
    throw new TypeError("XcelerateError values must be instances of XcelerateErrorWsError, XcelerateErrorSerdeError, XcelerateErrorCdpResponseError, XcelerateErrorHttpError, XcelerateErrorNotFound, XcelerateErrorInternalError, XcelerateErrorUnsupported.");
  }

  read(reader) {
    const enumTag = reader.readInt32();
    switch (enumTag) {
      case 1:
        return new XcelerateErrorWsError(FfiConverterString.read(reader));
      case 2:
        return new XcelerateErrorSerdeError(FfiConverterString.read(reader));
      case 3:
        return new XcelerateErrorCdpResponseError(FfiConverterString.read(reader));
      case 4:
        return new XcelerateErrorHttpError(FfiConverterString.read(reader));
      case 5:
        return new XcelerateErrorNotFound(FfiConverterString.read(reader));
      case 6:
        return new XcelerateErrorInternalError(FfiConverterString.read(reader));
      case 7:
        return new XcelerateErrorUnsupported(FfiConverterString.read(reader));
      default:
        throw new UnexpectedEnumCase(`Unexpected XcelerateError case ${String(enumTag)}.`);
    }
  }
})();

const uniffiRegisteredCallbackPointers = [];
const uniffiRegisteredCallbackVtables = [];

function uniffiRegisterCallbackVtables(bindings) {
}

function uniffiUnregisterCallbackVtables() {
  clearPendingForeignFutures();
  while (uniffiRegisteredCallbackPointers.length > 0) {
    koffi.unregister(uniffiRegisteredCallbackPointers.pop());
  }
  uniffiRegisteredCallbackVtables.length = 0;
}

configureRuntimeHooks({
  onLoad(bindings) {
    void bindings;
  },
  onUnload() {
    if (uniffiRustFutureContinuationPointer != null) {
      koffi.unregister(uniffiRustFutureContinuationPointer);
      uniffiRustFutureContinuationPointer = null;
    }
    uniffiUnregisterCallbackVtables();
  },
});

/**
 * Represents a browser instance (e.g., Chrome or Edge).
 */
export class Browser extends UniffiObjectBase {
  constructor() {
    super();
    return uniffiNotImplemented("Browser.constructor");
  }

  static async launch(config = {}) {
    const finalConfig = {
      headless: true,
      stealth: false,
      detached: true,
      executable_path: null,
      plugins: null,
      ...config
    };
    config = finalConfig;
    const loweredConfig = uniffiLowerIntoRustBuffer(FfiConverterBrowserConfig, config);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiFunctions.uniffi_xcelerate_fn_constructor_browser_launch(loweredConfig),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      completeFunc,
      liftFunc: (pointer) => uniffiBrowserObjectFactory.createRawExternal(pointer),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the plugin audit log as a JSON array (no secrets are recorded).
   */
  auditLog() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_log_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_log;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftStringFromRustBuffer(uniffiResult);
  }

  /**
   * Verifies the integrity of the append-only plugin audit log.
   */
  auditVerify() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_verify_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_audit_verify;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return FfiConverterBool.lift(uniffiResult);
  }

  /**
   * Names of all compiled-in first-party plugins (the catalog).
   */
  availablePlugins() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_available_plugins_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_available_plugins;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterString), uniffiResult);
  }

  /**
   * Returns the browser context ids as a JSON array.
   */
  async browser_contexts() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_browser_contexts_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_browser_contexts;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the browser version info as JSON.
   */
  async capabilities() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_capabilities_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_capabilities;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Closes the browser and kills the process.
   */
  async close() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_close_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_close;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns all browser cookies as a JSON array.
   */
  async cookies() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_cookies_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_cookies;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Deletes cookies with the given name.
   */
  async delete_cookie(name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_delete_cookie_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_delete_cookie;
    const loweredName = uniffiLowerString(name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the registered event names.
   */
  async event_names() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_event_names_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_event_names;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterString), uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Grants permissions (JSON array) to an origin.
   */
  async grant_permissions(origin, permissions_json) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_grant_permissions_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_grant_permissions;
    const loweredOrigin = uniffiLowerString(origin);
    const loweredPermissionsJson = uniffiLowerString(permissions_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredOrigin, loweredPermissionsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Whether the underlying connection is alive.
   */
  async is_connected() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_is_connected_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_is_connected;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Whether an event name is registered.
   */
  async listens_to(event_name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_listens_to_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_listens_to;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Loads a third-party plugin. Not supported in this phase.
   *
   * The sandboxed, out-of-process runner required for untrusted plugins does
   * not exist yet, so this always refuses rather than executing unknown code.
   */
  loadPlugin(path) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_load_plugin_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_load_plugin;
    const loweredPath = uniffiLowerString(path);
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, loweredPath, status),
      uniffiRustCallOptions(FfiConverterXcelerateError),
    );
    return uniffiLiftStringFromRustBuffer(uniffiResult);
  }

  /**
   * Creates a new (incognito) browser context and returns its id.
   */
  async new_context() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_new_context_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_new_context;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async newPage(url) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_new_page_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_new_page;
    const loweredUrl = uniffiLowerString(url);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUrl),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Registers interest in a root-session CDP event.
   */
  async on(event_name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_on_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_on;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Alias for [`Browser::on`].
   */
  async once(event_name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_once_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_once;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Returns a handle to an enabled plugin so its ops can be invoked.
   */
  plugin(name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin;
    const loweredName = uniffiLowerString(name);
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, loweredName, status),
      uniffiRustCallOptions(FfiConverterXcelerateError),
    );
    return uniffiPluginHandleObjectFactory.create(uniffiResult);
  }

  /**
   * Names of the plugins currently enabled on this browser.
   */
  pluginNames() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_names_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_plugin_names;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterString), uniffiResult);
  }

  /**
   * Removes every registered listener.
   */
  async remove_all_listeners() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_all_listeners_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_all_listeners;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Removes a single registered listener.
   */
  async remove_listener(event_name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_listener_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_remove_listener;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Resets all permission overrides.
   */
  async reset_permissions() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_reset_permissions_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_reset_permissions;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets a cookie from a JSON object.
   */
  async set_cookie(cookie_json) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_set_cookie_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_set_cookie;
    const loweredCookieJson = uniffiLowerString(cookie_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredCookieJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets the download directory for the browser.
   */
  async set_download_behavior(path) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_set_download_behavior_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_set_download_behavior;
    const loweredPath = uniffiLowerString(path);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPath),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Starts CDP tracing.
   */
  async start_tracing() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_start_tracing_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_start_tracing;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stops CDP tracing.
   */
  async stop_tracing() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_stop_tracing_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_stop_tracing;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the current targets as a JSON array (`Target.getTargets`).
   */
  async targets() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_targets_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_targets;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Enables a compiled-in first-party plugin at runtime.
   *
   * Launch-time contributions (such as binary patching) only take effect if
   * the plugin was enabled before the browser launched; enabling a plugin
   * afterwards applies its runtime hooks to pages created from now on. This
   * is audited as a runtime enable. Unknown or third-party names are refused.
   */
  async usePlugin(name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_use_plugin_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_use_plugin;
    const loweredName = uniffiLowerString(name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the browser's user agent.
   */
  async user_agent() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_user_agent_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_user_agent;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the browser version information.
   */
  async version() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_version_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_version;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for the next root-session CDP event named `event_name`.
   */
  async wait_for_event(event_name, timeout_ms) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event;
    const loweredEventName = uniffiLowerString(event_name);
    const loweredTimeoutMs = FfiConverterUInt64.lower(timeout_ms);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName, loweredTimeoutMs),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * [`Browser::wait_for_event`] with the default 30s timeout.
   */
  async wait_for_event_default(event_name) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_default_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_wait_for_event_default;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * The WebSocket endpoint Chrome was launched with.
   */
  ws_endpoint() {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_browser_ws_endpoint_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_browser_ws_endpoint;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftStringFromRustBuffer(uniffiResult);
  }
}

const uniffiBrowserObjectFactory = createObjectFactory({
  typeName: "Browser",
  createInstance: () => Object.create(Browser.prototype),
  cloneFreeUsesUniffiHandle: true,
  cloneHandleGeneric(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_browser_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_clone_browser:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_clone_browser",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    return uniffiRustCaller.rustCall(
      (status) => rawExternalCloneHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandle(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_browser(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_browser_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_free_browser:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_free_browser",
        "void",
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    uniffiRustCaller.rustCall(
      (status) => rawExternalFreeHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandle(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_browser(handle, status),
      uniffiRustCallOptions(),
    );
  },
});
const FfiConverterBrowser = createObjectConverter(uniffiBrowserObjectFactory);

/**
 * Represents an HTML element in the DOM.
 */
export class Element extends UniffiObjectBase {
  constructor() {
    super();
    return uniffiNotImplemented("Element.constructor");
  }

  /**
   * Returns the value of a specific attribute.
   */
  async attribute(name) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_attribute_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_attribute;
    const loweredName = uniffiLowerString(name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiOptionalConverter(FfiConverterString), uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Like [`Element::call_json`] but coerces the result to a bool.
   */
  async call_bool(function_, args_json) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_call_bool_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_call_bool;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a JS function on this element with JSON-encoded arguments.
   */
  async call_json(function_, args_json) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_call_json_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_call_json;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Runs a JS function against the first descendant matching `selector`.
   */
  async call_on_selector(selector, expression) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector;
    const loweredSelector = uniffiLowerString(selector);
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Runs a JS function against every descendant matching `selector`.
   */
  async call_on_selector_all(selector, expression) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_all_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_call_on_selector_all;
    const loweredSelector = uniffiLowerString(selector);
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Like [`Element::call_json`] but coerces the result to a string.
   */
  async call_string(function_, args_json) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_call_string_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_call_string;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Clicks the element.
   */
  async click() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_click_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_click;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Clicks the element using realistic mouse movement and CDP input events.
   */
  async click_stealth() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_click_stealth_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_click_stealth;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Number of elements this handle represents (always 1).
   */
  async count() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_count_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_count;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i64(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i64(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterInt64.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Releases the underlying remote object handle.
   */
  async dispose() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_dispose_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_dispose;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a function on this element and coerces the result to a bool.
   */
  async evaluate_bool(function_) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_bool_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_bool;
    const loweredFunction = uniffiLowerString(function_);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a JS function on this element and returns the resulting node.
   */
  async evaluate_handle(function_) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_handle_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_handle;
    const loweredFunction = uniffiLowerString(function_);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a function on this element; the result is returned as JSON text.
   */
  async evaluate_json(function_) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_json_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_json;
    const loweredFunction = uniffiLowerString(function_);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a function on this element and coerces the result to a string.
   */
  async evaluate_string(function_) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_string_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_evaluate_string;
    const loweredFunction = uniffiLowerString(function_);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Focuses the element.
   */
  async focus() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_focus_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_focus;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a descendant form control by its `<label>` text.
   */
  async get_by_label(label) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_label_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_label;
    const loweredLabel = uniffiLowerString(label);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredLabel),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a descendant by ARIA role.
   */
  async get_by_role(role) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_role_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_role;
    const loweredRole = uniffiLowerString(role);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredRole),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a descendant whose text contains `text`.
   */
  async get_by_text(text) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_get_by_text;
    const loweredText = uniffiLowerString(text);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns this element's enumerable properties as a JSON object.
   */
  async get_properties() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_get_properties_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_get_properties;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Hovers over the element.
   */
  async hover() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_hover_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_hover;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Hovers over the element using realistic mouse movement.
   */
  async hover_stealth() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_hover_stealth_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_hover_stealth;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the inner HTML of the element.
   */
  async innerHtml() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_inner_html_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_inner_html;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Focuses the element and presses a key.
   */
  async press(key) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_press_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_press;
    const loweredKey = uniffiLowerString(key);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredKey),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the first descendant matching `selector` as an [`Element`].
   */
  async query_selector(selector) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns every descendant matching `selector`.
   *
   * Resolves the whole node list with a single `Runtime.getProperties` call
   * rather than one `evaluate` per match.
   */
  async query_selector_all(selector) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_all_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_all;
    const loweredSelector = uniffiLowerString(selector);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterElement), uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a descendant by attribute value.
   */
  async query_selector_attr(attribute, value) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_attr_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_attr;
    const loweredAttribute = uniffiLowerString(attribute);
    const loweredValue = uniffiLowerString(value);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredAttribute, loweredValue),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a descendant matching an XPath expression.
   */
  async query_selector_xpath(xpath) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_xpath_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_query_selector_xpath;
    const loweredXpath = uniffiLowerString(xpath);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredXpath),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Captures a PNG screenshot cropped to this element.
   */
  async screenshot() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Captures a base64 PNG screenshot cropped to this element.
   */
  async screenshot_base64() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_base64_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_screenshot_base64;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Selects options by value or label on this `<select>` element.
   */
  async select_option(values_json) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_select_option_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_select_option;
    const loweredValuesJson = uniffiLowerString(values_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredValuesJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets the files of this `<input type="file">` element.
   */
  async set_input_files(files_json) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_set_input_files_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_set_input_files;
    const loweredFilesJson = uniffiLowerString(files_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFilesJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the visible text of the element.
   */
  async text() {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_text;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async typeText(text) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_type_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_type_text;
    const loweredText = uniffiLowerString(text);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for a descendant matching `selector` to appear.
   */
  async waitForSelector(selector) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_element_wait_for_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_element_wait_for_selector;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }
}

const uniffiElementObjectFactory = createObjectFactory({
  typeName: "Element",
  createInstance: () => Object.create(Element.prototype),
  cloneFreeUsesUniffiHandle: true,
  cloneHandleGeneric(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_element_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_clone_element:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_clone_element",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    return uniffiRustCaller.rustCall(
      (status) => rawExternalCloneHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandle(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_element(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_element_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_free_element:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_free_element",
        "void",
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    uniffiRustCaller.rustCall(
      (status) => rawExternalFreeHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandle(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_element(handle, status),
      uniffiRustCallOptions(),
    );
  },
});
const FfiConverterElement = createObjectConverter(uniffiElementObjectFactory);

export class Page extends UniffiObjectBase {
  constructor() {
    super();
    return uniffiNotImplemented("Page.constructor");
  }

  /**
   * Activates this page's target.
   */
  async activate() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_activate_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_activate;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Activates the given target (window/tab).
   */
  async activate_target(target_id) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_activate_target_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_activate_target;
    const loweredTargetId = uniffiLowerString(target_id);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredTargetId),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Evaluates a script on every new document.
   */
  async addScriptToEvaluateOnNewDocument(source) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document;
    const loweredSource = uniffiLowerString(source);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSource),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Injects a `<style>` element and returns the injected content.
   */
  async add_style_tag(content) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_add_style_tag_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_add_style_tag;
    const loweredContent = uniffiLowerString(content);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredContent),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets the credentials used to answer HTTP auth challenges.
   */
  async authenticate(username, password) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_authenticate_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_authenticate;
    const loweredUsername = uniffiLowerString(username);
    const loweredPassword = uniffiLowerString(password);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUsername, loweredPassword),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Brings the page to the front.
   */
  async bring_to_front() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_bring_to_front_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_bring_to_front;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Like [`Page::call_json`] but coerces the result to a bool.
   */
  async call_bool(function_, args_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_call_bool_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_call_bool;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Calls a JS function with JSON-encoded arguments, returning JSON text.
   *
   * This is the shim the adapters use to express the broad upstream surface
   * as data (a function body per method) rather than a core method per member.
   */
  async call_json(function_, args_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_call_json_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_call_json;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Runs a JS function against the element matching `selector` (`$eval`).
   */
  async call_on_selector(selector, expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector;
    const loweredSelector = uniffiLowerString(selector);
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Runs a JS function against every element matching `selector` (`$$eval`).
   */
  async call_on_selector_all(selector, expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_all_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_call_on_selector_all;
    const loweredSelector = uniffiLowerString(selector);
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Like [`Page::call_json`] but coerces the result to a string.
   */
  async call_string(function_, args_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_call_string_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_call_string;
    const loweredFunction = uniffiLowerString(function_);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFunction, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Clears the recorded intercepted requests.
   */
  async clear_requests() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_clear_requests_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_clear_requests;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
   */
  async click_mouse(x, y) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_click_mouse_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_click_mouse;
    const loweredX = FfiConverterFloat64.lower(x);
    const loweredY = FfiConverterFloat64.lower(y);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredX, loweredY),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Closes the page.
   */
  async close() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_close_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_close;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the full HTML content of the page.
   */
  async content() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_content_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_content;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns a single cookie by name as JSON (or null).
   */
  async cookie(name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_cookie_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_cookie;
    const loweredName = uniffiLowerString(name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the cookies visible to this page as a JSON array.
   */
  async cookies() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_cookies_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_cookies;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Starts CSS coverage collection.
   */
  async coverage_start_css() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_css_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_css;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Starts JS coverage collection.
   */
  async coverage_start_js() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_js_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_start_js;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stops CSS coverage collection and returns the result as JSON.
   */
  async coverage_stop_css() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_css_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_css;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stops JS coverage collection and returns the result as JSON.
   */
  async coverage_stop_js() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_js_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_coverage_stop_js;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the page PDF as a base64 string.
   */
  async create_pdf_stream() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_create_pdf_stream_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_create_pdf_stream;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  decode_base64(data) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_decode_base64_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_decode_base64;
    const loweredData = uniffiLowerString(data);
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, loweredData, status),
      uniffiRustCallOptions(FfiConverterXcelerateError),
    );
    return uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult);
  }

  /**
   * Overrides the idle state.
   */
  async emulate_idle_state(is_user_active, is_screen_unlocked) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_idle_state_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_idle_state;
    const loweredIsUserActive = FfiConverterBool.lower(is_user_active);
    const loweredIsScreenUnlocked = FfiConverterBool.lower(is_screen_unlocked);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredIsUserActive, loweredIsScreenUnlocked),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Emulates a media type and/or colour scheme.
   */
  async emulate_media(media, color_scheme) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_media_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_emulate_media;
    const loweredMedia = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), media);
    const loweredColorScheme = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), color_scheme);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredMedia, loweredColorScheme),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async ensure_interception() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_ensure_interception_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_ensure_interception;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Evaluates JavaScript and coerces the result to a bool.
   */
  async evaluate_bool(expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_bool_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_bool;
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Evaluates JavaScript and returns the resulting object as an [`Element`].
   */
  async evaluate_handle(expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_handle_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_handle;
    const loweredExpression = uniffiLowerString(expression);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Evaluates JavaScript in the page and returns the result as a JSON string.
   */
  async evaluate_json(expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_json_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_json;
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Evaluates JavaScript and coerces the result to a string.
   */
  async evaluate_string(expression) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_string_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_evaluate_string;
    const loweredExpression = uniffiLowerString(expression);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredExpression),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the registered event names.
   */
  async event_names() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_event_names_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_event_names;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterString), uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Escape hatch: sends an arbitrary CDP command and returns its JSON result.
   *
   * The adapters use this to express CDP-backed library methods as data
   * (a method name + parameter template), keeping the core surface small.
   */
  async execute_cdp_cmd(method, params_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_execute_cdp_cmd_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_execute_cdp_cmd;
    const loweredMethod = uniffiLowerString(method);
    const loweredParamsJson = uniffiLowerString(params_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredMethod, loweredParamsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds an element matching the CSS selector.
   */
  async findElement(selector) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_find_element_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_find_element;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the frame matching an id or name as JSON (or null).
   */
  async frame(frame_id) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_frame_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_frame;
    const loweredFrameId = uniffiLowerString(frame_id);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFrameId),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the main frame's name.
   */
  async frame_name() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_frame_name_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_frame_name;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns every frame in the page as a JSON array.
   */
  async frames() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_frames_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_frames;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds a form control by its associated `<label>` text.
   */
  async get_by_label(label) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_label_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_label;
    const loweredLabel = uniffiLowerString(label);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredLabel),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds an element by ARIA role (falls back to a tag-name lookup).
   */
  async get_by_role(role) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_role_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_role;
    const loweredRole = uniffiLowerString(role);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredRole),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Finds an element whose text content contains `text`.
   */
  async get_by_text(text) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_get_by_text;
    const loweredText = uniffiLowerString(text);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the stored default timeout (ms).
   */
  async get_default_timeout() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_get_default_timeout_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_get_default_timeout;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_f64(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_f64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_f64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_f64(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterFloat64.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async go_back() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_go_back_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_go_back;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Navigates forward in history.
   */
  async go_forward() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_go_forward_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_go_forward;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Accepts or dismisses the active JavaScript dialog.
   */
  async handle_js_dialog(accept, prompt_text) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_handle_js_dialog_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_handle_js_dialog;
    const loweredAccept = FfiConverterBool.lower(accept);
    const loweredPromptText = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), prompt_text);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredAccept, loweredPromptText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Reads a local file and injects it as an init script.
   */
  async inject_file(path) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_inject_file_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_inject_file;
    const loweredPath = uniffiLowerString(path);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPath),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Whether drag interception is enabled.
   */
  async is_drag_interception_enabled() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Dispatches a keydown event for `key`.
   */
  async keyboard_down(key) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_down_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_down;
    const loweredKey = uniffiLowerString(key);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredKey),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Presses `key` on the focused element.
   */
  async keyboard_press(key) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_press_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_press;
    const loweredKey = uniffiLowerString(key);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredKey),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Types `text` into the focused element.
   */
  async keyboard_type(text) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_type_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_type;
    const loweredText = uniffiLowerString(text);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Dispatches a keyup event for `key`.
   */
  async keyboard_up(key) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_up_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_keyboard_up;
    const loweredKey = uniffiLowerString(key);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredKey),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Whether an event name is registered.
   */
  async listens_to(event_name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_listens_to_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_listens_to;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i8(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i8(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i8(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i8(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterBool.lift(uniffiResult),
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Returns the main frame as JSON.
   */
  async main_frame() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_main_frame_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_main_frame;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the page performance metrics as a JSON object.
   */
  async metrics() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_metrics_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_metrics;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Triggers a mousePress event at the current mouse coordinates.
   */
  async mouse_down(button) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_down_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_down;
    const loweredButton = uniffiLowerString(button);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredButton),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Triggers a mouseReleased event at the current mouse coordinates.
   */
  async mouse_up(button) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_up_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_mouse_up;
    const loweredButton = uniffiLowerString(button);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredButton),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
   */
  async move_mouse(x, y) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_move_mouse_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_move_mouse;
    const loweredX = FfiConverterFloat64.lower(x);
    const loweredY = FfiConverterFloat64.lower(y);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredX, loweredY),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Navigates to a URL.
   */
  async navigate(url) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_navigate_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_navigate;
    const loweredUrl = uniffiLowerString(url);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUrl),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Registers interest in a CDP event name.
   */
  async on(event_name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_on_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_on;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Alias for [`Page::on`].
   */
  async once(event_name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_once_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_once;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  async pdf() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_pdf_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_pdf;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Focuses the element matching `selector` and presses `key`.
   */
  async press(selector, key) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_press_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_press;
    const loweredSelector = uniffiLowerString(selector);
    const loweredKey = uniffiLowerString(key);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredKey),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns every element matching the CSS selector.
   *
   * Uses two round trips (fetch the node list, then read its properties)
   * instead of one `evaluate` per match.
   */
  async query_selector_all(selector) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_all_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_all;
    const loweredSelector = uniffiLowerString(selector);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterElement), uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the first node matching an XPath expression as an [`Element`].
   */
  async query_selector_xpath(xpath) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_xpath_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_query_selector_xpath;
    const loweredXpath = uniffiLowerString(xpath);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredXpath),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async raw_window_bounds() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_raw_window_bounds_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_raw_window_bounds;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Reloads the page.
   */
  async reload() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_reload_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_reload;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Removes every registered event listener.
   */
  async remove_all_listeners() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_remove_all_listeners_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_remove_all_listeners;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Removes a single registered event listener.
   */
  async remove_listener(event_name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_remove_listener_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_remove_listener;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(),
    });
  }

  /**
   * Removes an init script by identifier.
   */
  async remove_script(identifier) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_remove_script_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_remove_script;
    const loweredIdentifier = uniffiLowerString(identifier);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredIdentifier),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the most recent intercepted request as JSON.
   */
  async request() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_request_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_request;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the intercepted requests seen so far as JSON.
   */
  async requests() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_requests_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_requests;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
   */
  async route(pattern, action, body, content_type) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_route_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_route;
    const loweredPattern = uniffiLowerString(pattern);
    const loweredAction = uniffiLowerString(action);
    const loweredBody = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), body);
    const loweredContentType = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), content_type);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPattern, loweredAction, loweredBody, loweredContentType),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Aborts every request matching `pattern`.
   */
  async route_abort(pattern) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_route_abort_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_route_abort;
    const loweredPattern = uniffiLowerString(pattern);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPattern),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Registers fulfill routes for every entry in a HAR file.
   */
  async route_from_har(path) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_route_from_har_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_route_from_har;
    const loweredPath = uniffiLowerString(path);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPath),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Fulfills every request matching `pattern` with `body`.
   */
  async route_fulfill(pattern, body, content_type) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_route_fulfill_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_route_fulfill;
    const loweredPattern = uniffiLowerString(pattern);
    const loweredBody = uniffiLowerString(body);
    const loweredContentType = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), content_type);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPattern, loweredBody, loweredContentType),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async screenshot() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async screenshotFull() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_full_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_screenshot_full;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Selects options by value/label on the matching `<select>`.
   */
  async select_option(selector, values_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_select_option_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_select_option;
    const loweredSelector = uniffiLowerString(selector);
    const loweredValuesJson = uniffiLowerString(values_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredValuesJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Enables or disables the HTTP cache.
   */
  async set_cache_enabled(enabled) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_cache_enabled_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_cache_enabled;
    const loweredEnabled = FfiConverterBool.lower(enabled);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEnabled),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Replaces the document content.
   */
  async set_content(html) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_content_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_content;
    const loweredHtml = uniffiLowerString(html);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredHtml),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stores a default timeout (ms) for adapter compatibility.
   */
  async set_default_timeout(milliseconds) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_default_timeout_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_default_timeout;
    const loweredMilliseconds = FfiConverterFloat64.lower(milliseconds);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredMilliseconds),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Enables or disables input drag interception.
   */
  async set_drag_interception(enabled) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_drag_interception_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_drag_interception;
    const loweredEnabled = FfiConverterBool.lower(enabled);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEnabled),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Overrides media features (JSON array of `{name,value}`).
   */
  async set_emulated_media_features(features_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_emulated_media_features_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_emulated_media_features;
    const loweredFeaturesJson = uniffiLowerString(features_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredFeaturesJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets extra HTTP headers for every request from this page.
   */
  async set_extra_http_headers(headers_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_extra_http_headers_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_extra_http_headers;
    const loweredHeadersJson = uniffiLowerString(headers_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredHeadersJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets the files of the matching `<input type="file">`.
   */
  async set_input_files(selector, files_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_input_files_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_input_files;
    const loweredSelector = uniffiLowerString(selector);
    const loweredFilesJson = uniffiLowerString(files_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector, loweredFilesJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Enables or disables JavaScript execution.
   */
  async set_javascript_enabled(enabled) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_javascript_enabled_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_javascript_enabled;
    const loweredEnabled = FfiConverterBool.lower(enabled);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEnabled),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Toggles offline mode.
   */
  async set_offline(offline) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_offline_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_offline;
    const loweredOffline = FfiConverterBool.lower(offline);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredOffline),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Enables or disables request interception for this page.
   */
  async set_request_interception(enabled) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_request_interception_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_request_interception;
    const loweredEnabled = FfiConverterBool.lower(enabled);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEnabled),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Restores cookies + localStorage from a storage-state JSON object.
   */
  async set_storage_state(state_json) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_storage_state_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_storage_state;
    const loweredStateJson = uniffiLowerString(state_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredStateJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Overrides `navigator.userAgent` for this page.
   */
  async set_user_agent(user_agent, accept_language) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_user_agent_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_user_agent;
    const loweredUserAgent = uniffiLowerString(user_agent);
    const loweredAcceptLanguage = uniffiLowerIntoRustBuffer(uniffiOptionalConverter(FfiConverterString), accept_language);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUserAgent, loweredAcceptLanguage),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Overrides the viewport size.
   */
  async set_viewport_size(width, height) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_viewport_size_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_viewport_size;
    const loweredWidth = FfiConverterUInt64.lower(width);
    const loweredHeight = FfiConverterInt64.lower(height);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredWidth, loweredHeight),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Moves and resizes the window.
   */
  async set_window_bounds(left, top, width, height) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_bounds_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_bounds;
    const loweredLeft = FfiConverterInt64.lower(left);
    const loweredTop = FfiConverterInt64.lower(top);
    const loweredWidth = FfiConverterInt64.lower(width);
    const loweredHeight = FfiConverterInt64.lower(height);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredLeft, loweredTop, loweredWidth, loweredHeight),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Moves the window, preserving its size.
   */
  async set_window_position(x, y) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_position_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_position;
    const loweredX = FfiConverterInt64.lower(x);
    const loweredY = FfiConverterInt64.lower(y);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredX, loweredY),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Resizes the window, preserving its position.
   */
  async set_window_size(width, height) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_size_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_size;
    const loweredWidth = FfiConverterInt64.lower(width);
    const loweredHeight = FfiConverterInt64.lower(height);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredWidth, loweredHeight),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
   */
  async set_window_state(state) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_state_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_set_window_state;
    const loweredState = uniffiLowerString(state);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredState),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Starts a PNG screencast.
   */
  async start_screencast() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_start_screencast_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_start_screencast;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Starts CDP tracing on this page's session.
   */
  async start_tracing() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_start_tracing_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_start_tracing;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stops the screencast.
   */
  async stop_screencast() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_stop_screencast_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_stop_screencast;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Stops CDP tracing on this page's session.
   */
  async stop_tracing() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_stop_tracing_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_stop_tracing;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns cookies + localStorage as a storage-state JSON object.
   */
  async storage_state() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_storage_state_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_storage_state;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * The CDP target id backing this page.
   */
  target_id() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_target_id_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_target_id;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftStringFromRustBuffer(uniffiResult);
  }

  /**
   * Returns the page title.
   */
  async title() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_title_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_title;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Dispatches a touch tap at (x, y).
   */
  async touch_tap(x, y) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_touch_tap_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_touch_tap;
    const loweredX = FfiConverterFloat64.lower(x);
    const loweredY = FfiConverterFloat64.lower(y);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredX, loweredY),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Removes the routes registered for `pattern`.
   */
  async unroute(pattern) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_unroute;
    const loweredPattern = uniffiLowerString(pattern);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredPattern),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Removes every route rule.
   */
  async unroute_all() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_all_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_unroute_all;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the current document URL.
   */
  async url() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_url_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_url;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for the next CDP event named `event_name` and returns its params.
   *
   * The relevant domain is enabled first (best effort), so callers do not
   * have to.
   */
  async wait_for_event(event_name, timeout_ms) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event;
    const loweredEventName = uniffiLowerString(event_name);
    const loweredTimeoutMs = FfiConverterUInt64.lower(timeout_ms);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName, loweredTimeoutMs),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * [`Page::wait_for_event`] with the default 30s timeout.
   */
  async wait_for_event_default(event_name) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_default_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_event_default;
    const loweredEventName = uniffiLowerString(event_name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredEventName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
   */
  async wait_for_function(expression, timeout_ms) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_function_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_function;
    const loweredExpression = uniffiLowerString(expression);
    const loweredTimeoutMs = FfiConverterUInt64.lower(timeout_ms);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredExpression, loweredTimeoutMs),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for the page to finish loading.
   */
  async waitForNavigation() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_navigation_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_navigation;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for an element matching the selector to appear in the DOM.
   */
  async waitForSelector(selector) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_selector;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Waits for the first XPath match to appear.
   */
  async wait_for_xpath(xpath, timeout_ms) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_xpath_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_wait_for_xpath;
    const loweredXpath = uniffiLowerString(xpath);
    const loweredTimeoutMs = FfiConverterUInt64.lower(timeout_ms);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredXpath, loweredTimeoutMs),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async window_id() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_window_id_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_window_id;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_i64(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_i64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_i64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_i64(rustFuture),
      liftFunc: (uniffiResult) => FfiConverterInt64.lift(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the window position as JSON (`{x,y}`).
   */
  async window_position() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_window_position_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_window_position;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
   */
  async window_rect() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_window_rect_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_window_rect;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the window size as JSON (`{width,height}`).
   */
  async window_size() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_page_window_size_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_page_window_size;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }
}

const uniffiPageObjectFactory = createObjectFactory({
  typeName: "Page",
  createInstance: () => Object.create(Page.prototype),
  cloneFreeUsesUniffiHandle: true,
  cloneHandleGeneric(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_page_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_clone_page:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_clone_page",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    return uniffiRustCaller.rustCall(
      (status) => rawExternalCloneHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandle(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_page(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_page_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_free_page:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_free_page",
        "void",
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    uniffiRustCaller.rustCall(
      (status) => rawExternalFreeHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandle(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_page(handle, status),
      uniffiRustCallOptions(),
    );
  },
});
const FfiConverterPage = createObjectConverter(uniffiPageObjectFactory);

/**
 * A handle to an enabled plugin, exposed to every language.
 */
export class PluginHandle extends UniffiObjectBase {
  constructor() {
    super();
    return uniffiNotImplemented("PluginHandle.constructor");
  }

  /**
   * Invoke an op with a JSON-encoded argument object; returns JSON.
   */
  async invoke(op, args_json) {
    const loweredSelf = uniffiPluginHandleObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPluginHandleObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_invoke_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_invoke;
    const loweredOp = uniffiLowerString(op);
    const loweredArgsJson = uniffiLowerString(args_json);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredOp, loweredArgsJson),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * The ops this plugin exposes.
   */
  ops() {
    const loweredSelf = uniffiPluginHandleObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPluginHandleObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_ops_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_ops;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftFromRustBuffer(uniffiArrayConverter(FfiConverterString), uniffiResult);
  }

  /**
   * The plugin's name.
   */
  pluginName() {
    const loweredSelf = uniffiPluginHandleObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPluginHandleObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_plugin_name_generic_abi
        : ffiFunctions.uniffi_xcelerate_fn_method_pluginhandle_plugin_name;
    const uniffiResult = uniffiRustCaller.rustCall(
      (status) => ffiMethod(loweredSelf, status),
      uniffiRustCallOptions(),
    );
    return uniffiLiftStringFromRustBuffer(uniffiResult);
  }
}

const uniffiPluginHandleObjectFactory = createObjectFactory({
  typeName: "PluginHandle",
  createInstance: () => Object.create(PluginHandle.prototype),
  cloneFreeUsesUniffiHandle: true,
  cloneHandleGeneric(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_pluginhandle_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_clone_pluginhandle:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_clone_pluginhandle",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    return uniffiRustCaller.rustCall(
      (status) => rawExternalCloneHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandle(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_clone_pluginhandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_pluginhandle_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_fn_free_pluginhandle:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_fn_free_pluginhandle",
        "void",
        [bindings.ffiTypes.VoidPointer, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    uniffiRustCaller.rustCall(
      (status) => rawExternalFreeHandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandle(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_fn_free_pluginhandle(handle, status),
      uniffiRustCallOptions(),
    );
  },
});
const FfiConverterPluginHandle = createObjectConverter(uniffiPluginHandleObjectFactory);