import koffi from "koffi";


import {

  configureRuntimeHooks,

  ffiFunctions,

  getFfiBindings,

  getFfiTypes,

} from "./xcelerate_core-ffi.js";


import {

  createForeignBytes,

  RustBufferValue,

} from "./runtime/ffi-types.js";


import {

  AbstractFfiConverterByteArray,

  FfiConverterArray,

  FfiConverterBool,

  FfiConverterBytes,

  FfiConverterOptional,

  FfiConverterString,

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
  namespace: "xcelerate_core",
  packageName: "xcelerate_core",
  cdylibName: "xcelerate_core",
  nodeEngine: ">=16",
  bundledPrebuilds: false,
  manualLoad: false,
});

export { ffiMetadata } from "./xcelerate_core-ffi.js";


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
    (status) => ffiFunctions.ffi_xcelerate_core_rustbuffer_free(buffer, status),
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
    (status) => ffiFunctions.ffi_xcelerate_core_rustbuffer_from_bytes(createForeignBytes(bytes), status),
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

const FfiConverterBrowserConfig = new (class extends AbstractFfiConverterByteArray {
  allocationSize(value) {
    const recordValue = uniffiRequireRecordObject("BrowserConfig", value);
    return FfiConverterBool.allocationSize(recordValue["headless"]) + FfiConverterBool.allocationSize(recordValue["stealth"]) + FfiConverterBool.allocationSize(recordValue["detached"]) + uniffiOptionalConverter(FfiConverterString).allocationSize(recordValue["executable_path"]);
  }

  write(value, writer) {
    const recordValue = uniffiRequireRecordObject("BrowserConfig", value);
    FfiConverterBool.write(recordValue["headless"], writer);
    FfiConverterBool.write(recordValue["stealth"], writer);
    FfiConverterBool.write(recordValue["detached"], writer);
    uniffiOptionalConverter(FfiConverterString).write(recordValue["executable_path"], writer);
  }

  read(reader) {
    return {
      "headless": FfiConverterBool.read(reader),
      "stealth": FfiConverterBool.read(reader),
      "detached": FfiConverterBool.read(reader),
      "executable_path": uniffiOptionalConverter(FfiConverterString).read(reader),
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
    throw new TypeError("XcelerateError values must be instances of XcelerateErrorWsError, XcelerateErrorSerdeError, XcelerateErrorCdpResponseError, XcelerateErrorHttpError, XcelerateErrorNotFound, XcelerateErrorInternalError.");
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
    throw new TypeError("XcelerateError values must be instances of XcelerateErrorWsError, XcelerateErrorSerdeError, XcelerateErrorCdpResponseError, XcelerateErrorHttpError, XcelerateErrorNotFound, XcelerateErrorInternalError.");
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
      stealth: true,
      detached: true,
      executable_path: null,
      ...config
    };
    config = finalConfig;
    const loweredConfig = uniffiLowerIntoRustBuffer(FfiConverterBrowserConfig, config);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiFunctions.uniffi_xcelerate_core_fn_constructor_browser_launch(loweredConfig),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      completeFunc,
      liftFunc: (pointer) => uniffiBrowserObjectFactory.createRawExternal(pointer),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_browser_close_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_browser_close;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async newPage(url) {
    const loweredSelf = uniffiBrowserObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiBrowserObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_browser_new_page_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_browser_new_page;
    const loweredUrl = uniffiLowerString(url);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUrl),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiPageObjectFactory.createRawExternal(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_browser_version_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_browser_version;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }
}

const uniffiBrowserObjectFactory = createObjectFactory({
  typeName: "Browser",
  createInstance: () => Object.create(Browser.prototype),
  cloneFreeUsesUniffiHandle: true,
  cloneHandleGeneric(handle) {
    return uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_browser_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_clone_browser:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_clone_browser",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_browser(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_browser_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_free_browser:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_free_browser",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_browser(handle, status),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_attribute_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_attribute;
    const loweredName = uniffiLowerString(name);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredName),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(uniffiOptionalConverter(FfiConverterString), uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_click_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_click;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_focus_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_focus;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_hover_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_hover;
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_inner_html_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_inner_html;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_text;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  async typeText(text) {
    const loweredSelf = uniffiElementObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiElementObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_element_type_text_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_element_type_text;
    const loweredText = uniffiLowerString(text);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredText),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_element_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_clone_element:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_clone_element",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_element(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_element_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_free_element:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_free_element",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_element(handle, status),
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
   * Evaluates a script on every new document.
   */
  async addScriptToEvaluateOnNewDocument(source) {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_add_script_to_evaluate_on_new_document;
    const loweredSource = uniffiLowerString(source);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSource),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_content_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_content;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_find_element_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_find_element;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Navigates back in history.
   */
  async go_back() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_go_back_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_go_back;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_navigate_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_navigate;
    const loweredUrl = uniffiLowerString(url);
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredUrl),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Captures a PDF of the page.
   */
  async pdf() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_pdf_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_pdf;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_reload_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_reload;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_void(rustFuture),
      liftFunc: (_uniffiResult) => undefined,
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Captures a screenshot of the page as a PNG.
   */
  async screenshot() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Captures a full-page screenshot by overriding device metrics.
   */
  async screenshotFull() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_full_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_screenshot_full;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftFromRustBuffer(FfiConverterBytes, uniffiResult),
      ...uniffiRustCallOptions(FfiConverterXcelerateError),
    });
  }

  /**
   * Returns the page title.
   */
  async title() {
    const loweredSelf = uniffiPageObjectFactory.cloneHandle(this);
    const ffiMethod =
      uniffiPageObjectFactory.usesGenericAbi(this)
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_title_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_title;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_rust_buffer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_rust_buffer(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_rust_buffer(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_rust_buffer(rustFuture),
      liftFunc: (uniffiResult) => uniffiLiftStringFromRustBuffer(uniffiResult),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_navigation_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_navigation;
    const completeFunc = (rustFuture, status) => ffiFunctions.ffi_xcelerate_core_rust_future_complete_void(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_void(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_void(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_void(rustFuture),
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
        ? ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_selector_generic_abi
        : ffiFunctions.uniffi_xcelerate_core_fn_method_page_wait_for_selector;
    const loweredSelector = uniffiLowerString(selector);
    const completePointer = uniffiGetCachedLibraryFunction(
      "complete:ffi_xcelerate_core_rust_future_complete_u64",
      (bindings) => bindings.library.func(
        "ffi_xcelerate_core_rust_future_complete_u64",
        bindings.ffiTypes.VoidPointer,
        [bindings.ffiTypes.UniffiHandle, koffi.pointer(bindings.ffiTypes.RustCallStatus)],
      ),
    );
    const completeFunc = (rustFuture, status) => completePointer(rustFuture, status);
    return rustCallAsync({
      rustFutureFunc: () => ffiMethod(loweredSelf, loweredSelector),
      pollFunc: (rustFuture, _continuationCallback, continuationHandle) => ffiFunctions.ffi_xcelerate_core_rust_future_poll_u64(rustFuture, uniffiGetRustFutureContinuationPointer(), continuationHandle),
      cancelFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_cancel_u64(rustFuture),
      completeFunc,
      freeFunc: (rustFuture) => ffiFunctions.ffi_xcelerate_core_rust_future_free_u64(rustFuture),
      liftFunc: (uniffiResult) => uniffiElementObjectFactory.createRawExternal(uniffiResult),
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_page_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  cloneHandleRawExternal(handle) {
    const rawExternalCloneHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_clone_page:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_clone_page",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_clone_page(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleGeneric(handle) {
    uniffiRustCaller.rustCall(
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_page_generic_abi(handle, status),
      uniffiRustCallOptions(),
    );
  },
  freeHandleRawExternal(handle) {
    const rawExternalFreeHandle = uniffiGetCachedLibraryFunction(
      "uniffi_xcelerate_core_fn_free_page:raw-external",
      (bindings) => bindings.library.func(
        "uniffi_xcelerate_core_fn_free_page",
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
      (status) => ffiFunctions.uniffi_xcelerate_core_fn_free_page(handle, status),
      uniffiRustCallOptions(),
    );
  },
});
const FfiConverterPage = createObjectConverter(uniffiPageObjectFactory);