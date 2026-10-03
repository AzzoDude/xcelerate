
package xcelerate

// #include <xcelerate.h>
import "C"

import (
	"bytes"
	"fmt"
	"io"
	"unsafe"
	"encoding/binary"
	"reflect"
	"runtime/cgo"
	"math"
	"runtime"
	"sync/atomic"
)



// This is needed, because as of go 1.24
// type RustBuffer C.RustBuffer cannot have methods,
// RustBuffer is treated as non-local type
type GoRustBuffer struct {
	inner C.RustBuffer
}

type RustBufferI interface {
	AsReader() *bytes.Reader
	Free()
	ToGoBytes() []byte
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

// C.RustBuffer fields exposed as an interface so they can be accessed in different Go packages.
// See https://github.com/golang/go/issues/13467
type ExternalCRustBuffer interface {
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

func RustBufferFromC(b C.RustBuffer) ExternalCRustBuffer {
	return GoRustBuffer {
		inner: b,
	}
}

func CFromRustBuffer(b ExternalCRustBuffer) C.RustBuffer {
	return C.RustBuffer {
		capacity: C.uint64_t(b.Capacity()),
		len: C.uint64_t(b.Len()),
		data: (*C.uchar)(b.Data()),
	}
}

func RustBufferFromExternal(b ExternalCRustBuffer) GoRustBuffer {
	return GoRustBuffer {
		inner: C.RustBuffer {
			capacity: C.uint64_t(b.Capacity()),
			len: C.uint64_t(b.Len()),
			data: (*C.uchar)(b.Data()),
		},
	}
}

func (cb GoRustBuffer) Capacity() uint64 {
	return uint64(cb.inner.capacity)
}

func (cb GoRustBuffer) Len() uint64 {
	return uint64(cb.inner.len)
}

func (cb GoRustBuffer) Data() unsafe.Pointer {
	return unsafe.Pointer(cb.inner.data)
}

func (cb GoRustBuffer) AsReader() *bytes.Reader {
	b := unsafe.Slice((*byte)(cb.inner.data), C.uint64_t(cb.inner.len))
	return bytes.NewReader(b)
}

func (cb GoRustBuffer) Free() {
	rustCall(func( status *C.RustCallStatus) bool {
		C.ffi_xcelerate_rustbuffer_free(cb.inner, status)
		return false
	})
}

func (cb GoRustBuffer) ToGoBytes() []byte {
	return C.GoBytes(unsafe.Pointer(cb.inner.data), C.int(cb.inner.len))
}


func stringToRustBuffer(str string) C.RustBuffer {
	return bytesToRustBuffer([]byte(str))
}

func bytesToRustBuffer(b []byte) C.RustBuffer {
	if len(b) == 0 {
		return C.RustBuffer{}
	}
	// We can pass the pointer along here, as it is pinned
	// for the duration of this call
	foreign := C.ForeignBytes {
		len: C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}
	
	return rustCall(func( status *C.RustCallStatus) C.RustBuffer {
		return C.ffi_xcelerate_rustbuffer_from_bytes(foreign, status)
	})
}


type BufLifter[GoType any] interface {
	Lift(value RustBufferI) GoType
}

type BufLowerer[GoType any] interface {
	Lower(value GoType) C.RustBuffer
}

type BufReader[GoType any] interface {
	Read(reader io.Reader) GoType
}

type BufWriter[GoType any] interface {
	Write(writer io.Writer, value GoType)
}

func LowerIntoRustBuffer[GoType any](bufWriter BufWriter[GoType], value GoType) C.RustBuffer {
	// This might be not the most efficient way but it does not require knowing allocation size
	// beforehand
	var buffer bytes.Buffer
	bufWriter.Write(&buffer, value)

	bytes, err := io.ReadAll(&buffer)
	if err != nil {
		panic(fmt.Errorf("reading written data: %w", err))
	}
	return bytesToRustBuffer(bytes)
}

func LiftFromRustBuffer[GoType any](bufReader BufReader[GoType], rbuf RustBufferI) GoType {
	defer rbuf.Free()
	reader := rbuf.AsReader()
	item := bufReader.Read(reader)
	if reader.Len() > 0 {
		// TODO: Remove this
		leftover, _ := io.ReadAll(reader)
		panic(fmt.Errorf("Junk remaining in buffer after lifting: %s", string(leftover)))
	}
	return item
}



func rustCallWithError[E any, U any](converter BufReader[E], callback func(*C.RustCallStatus) U) (U, E) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	err := checkCallStatus(converter, status)
	return returnValue, err
}

func checkCallStatus[E any](converter BufReader[E], status C.RustCallStatus) E {
	switch status.code {
	case 0:
		var zero E
		return zero
	case 1:
		return LiftFromRustBuffer(converter, GoRustBuffer { inner: status.errorBuf })
	case 2:
		// when the rust code sees a panic, it tries to construct a rustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer { inner: status.errorBuf })))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		panic(fmt.Errorf("unknown status code: %d", status.code))
	}
}

func checkCallStatusUnknown(status C.RustCallStatus) error {
	switch status.code {
	case 0:
		return nil
	case 1:
		panic(fmt.Errorf("function not returning an error returned an error"))
	case 2:
		// when the rust code sees a panic, it tries to construct a C.RustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer {
				inner: status.errorBuf,
			})))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return fmt.Errorf("unknown status code: %d", status.code)
	}
}

func rustCall[U any](callback func(*C.RustCallStatus) U) U {
	returnValue, err := rustCallWithError[error](nil, callback)
	if err != nil {
		panic(err)
	}
	return returnValue
}

type NativeError interface {
	AsError() error
}


func writeInt8(writer io.Writer, value int8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint8(writer io.Writer, value uint8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt16(writer io.Writer, value int16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint16(writer io.Writer, value uint16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt32(writer io.Writer, value int32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint32(writer io.Writer, value uint32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt64(writer io.Writer, value int64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint64(writer io.Writer, value uint64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat32(writer io.Writer, value float32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat64(writer io.Writer, value float64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}


func readInt8(reader io.Reader) int8 {
	var result int8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint8(reader io.Reader) uint8 {
	var result uint8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt16(reader io.Reader) int16 {
	var result int16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint16(reader io.Reader) uint16 {
	var result uint16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt32(reader io.Reader) int32 {
	var result int32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint32(reader io.Reader) uint32 {
	var result uint32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt64(reader io.Reader) int64 {
	var result int64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint64(reader io.Reader) uint64 {
	var result uint64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat32(reader io.Reader) float32 {
	var result float32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat64(reader io.Reader) float64 {
	var result float64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func init() {
        
        uniffiCheckChecksums()
}


func uniffiCheckChecksums() {
	// Get the bindings contract version from our ComponentInterface
	bindingsContractVersion := 30
	// Get the scaffolding contract version by calling the into the dylib
	scaffoldingContractVersion := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint32_t {
		return C.ffi_xcelerate_uniffi_contract_version()
	})
	if bindingsContractVersion != int(scaffoldingContractVersion) {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: UniFFI contract version mismatch")
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_audit_log()
	})
	if checksum != 37952 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_audit_log: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_audit_verify()
	})
	if checksum != 5412 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_audit_verify: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_available_plugins()
	})
	if checksum != 9431 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_available_plugins: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_browser_contexts()
	})
	if checksum != 50137 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_browser_contexts: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_capabilities()
	})
	if checksum != 7431 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_capabilities: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_close()
	})
	if checksum != 831 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_close: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_cookies()
	})
	if checksum != 36914 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_cookies: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_delete_cookie()
	})
	if checksum != 14366 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_delete_cookie: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_event_names()
	})
	if checksum != 44664 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_event_names: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_grant_permissions()
	})
	if checksum != 57820 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_grant_permissions: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_is_connected()
	})
	if checksum != 10958 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_is_connected: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_listens_to()
	})
	if checksum != 12245 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_listens_to: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_load_plugin()
	})
	if checksum != 5693 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_load_plugin: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_new_context()
	})
	if checksum != 28184 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_new_context: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_new_page()
	})
	if checksum != 31633 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_new_page: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_on()
	})
	if checksum != 2255 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_on: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_once()
	})
	if checksum != 22376 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_once: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_plugin()
	})
	if checksum != 11907 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_plugin: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_plugin_names()
	})
	if checksum != 58296 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_plugin_names: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_remove_all_listeners()
	})
	if checksum != 51158 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_remove_all_listeners: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_remove_listener()
	})
	if checksum != 41339 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_remove_listener: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_reset_permissions()
	})
	if checksum != 21496 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_reset_permissions: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_set_cookie()
	})
	if checksum != 6259 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_set_cookie: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_set_download_behavior()
	})
	if checksum != 23198 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_set_download_behavior: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_start_tracing()
	})
	if checksum != 14885 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_start_tracing: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_stop_tracing()
	})
	if checksum != 57049 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_stop_tracing: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_targets()
	})
	if checksum != 28936 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_targets: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_use_plugin()
	})
	if checksum != 20462 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_use_plugin: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_user_agent()
	})
	if checksum != 20558 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_user_agent: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_version()
	})
	if checksum != 64817 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_version: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_wait_for_event()
	})
	if checksum != 20884 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_wait_for_event: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_wait_for_event_default()
	})
	if checksum != 53096 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_wait_for_event_default: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_browser_ws_endpoint()
	})
	if checksum != 36520 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_browser_ws_endpoint: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_attribute()
	})
	if checksum != 8836 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_attribute: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_call_bool()
	})
	if checksum != 19329 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_call_bool: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_call_json()
	})
	if checksum != 56720 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_call_json: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_call_on_selector()
	})
	if checksum != 20589 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_call_on_selector: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_call_on_selector_all()
	})
	if checksum != 58660 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_call_on_selector_all: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_call_string()
	})
	if checksum != 1191 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_call_string: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_click()
	})
	if checksum != 26136 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_click: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_click_stealth()
	})
	if checksum != 64888 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_click_stealth: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_count()
	})
	if checksum != 40137 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_count: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_dispose()
	})
	if checksum != 27134 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_dispose: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_evaluate_bool()
	})
	if checksum != 62708 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_evaluate_bool: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_evaluate_handle()
	})
	if checksum != 34826 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_evaluate_handle: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_evaluate_json()
	})
	if checksum != 20134 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_evaluate_json: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_evaluate_string()
	})
	if checksum != 15210 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_evaluate_string: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_focus()
	})
	if checksum != 34225 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_focus: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_get_by_label()
	})
	if checksum != 41888 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_get_by_label: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_get_by_role()
	})
	if checksum != 15624 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_get_by_role: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_get_by_text()
	})
	if checksum != 11298 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_get_by_text: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_get_properties()
	})
	if checksum != 28646 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_get_properties: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_hover()
	})
	if checksum != 32638 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_hover: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_hover_stealth()
	})
	if checksum != 12397 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_hover_stealth: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_inner_html()
	})
	if checksum != 63319 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_inner_html: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_press()
	})
	if checksum != 13244 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_press: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_query_selector()
	})
	if checksum != 59248 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_query_selector: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_query_selector_all()
	})
	if checksum != 57750 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_query_selector_all: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_query_selector_attr()
	})
	if checksum != 63681 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_query_selector_attr: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_query_selector_xpath()
	})
	if checksum != 47775 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_query_selector_xpath: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_screenshot()
	})
	if checksum != 55082 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_screenshot: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_screenshot_base64()
	})
	if checksum != 62387 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_screenshot_base64: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_select_option()
	})
	if checksum != 21736 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_select_option: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_set_input_files()
	})
	if checksum != 32784 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_set_input_files: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_text()
	})
	if checksum != 41314 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_text: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_type_text()
	})
	if checksum != 45944 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_type_text: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_element_wait_for_selector()
	})
	if checksum != 53340 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_element_wait_for_selector: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_activate()
	})
	if checksum != 30852 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_activate: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_activate_target()
	})
	if checksum != 6362 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_activate_target: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document()
	})
	if checksum != 20123 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_add_style_tag()
	})
	if checksum != 9947 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_add_style_tag: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_authenticate()
	})
	if checksum != 15669 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_authenticate: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_bring_to_front()
	})
	if checksum != 14186 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_bring_to_front: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_call_bool()
	})
	if checksum != 62656 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_call_bool: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_call_json()
	})
	if checksum != 37033 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_call_json: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_call_on_selector()
	})
	if checksum != 27164 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_call_on_selector: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_call_on_selector_all()
	})
	if checksum != 24130 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_call_on_selector_all: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_call_string()
	})
	if checksum != 28160 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_call_string: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_clear_requests()
	})
	if checksum != 25306 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_clear_requests: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_click_mouse()
	})
	if checksum != 54243 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_click_mouse: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_close()
	})
	if checksum != 53159 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_close: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_content()
	})
	if checksum != 15096 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_content: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_cookie()
	})
	if checksum != 45979 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_cookie: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_cookies()
	})
	if checksum != 45327 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_cookies: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_coverage_start_css()
	})
	if checksum != 860 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_coverage_start_css: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_coverage_start_js()
	})
	if checksum != 4186 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_coverage_start_js: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_coverage_stop_css()
	})
	if checksum != 59476 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_coverage_stop_css: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_coverage_stop_js()
	})
	if checksum != 12341 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_coverage_stop_js: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_create_pdf_stream()
	})
	if checksum != 54525 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_create_pdf_stream: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_decode_base64()
	})
	if checksum != 39526 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_decode_base64: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_emulate_idle_state()
	})
	if checksum != 53017 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_emulate_idle_state: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_emulate_media()
	})
	if checksum != 27664 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_emulate_media: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_ensure_interception()
	})
	if checksum != 6857 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_ensure_interception: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_evaluate_bool()
	})
	if checksum != 12902 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_evaluate_bool: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_evaluate_handle()
	})
	if checksum != 57739 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_evaluate_handle: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_evaluate_json()
	})
	if checksum != 10653 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_evaluate_json: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_evaluate_string()
	})
	if checksum != 6817 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_evaluate_string: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_event_names()
	})
	if checksum != 15640 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_event_names: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_execute_cdp_cmd()
	})
	if checksum != 20070 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_execute_cdp_cmd: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_find_element()
	})
	if checksum != 4260 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_find_element: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_frame()
	})
	if checksum != 33986 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_frame: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_frame_name()
	})
	if checksum != 1668 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_frame_name: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_frames()
	})
	if checksum != 13809 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_frames: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_get_by_label()
	})
	if checksum != 63689 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_get_by_label: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_get_by_role()
	})
	if checksum != 32218 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_get_by_role: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_get_by_text()
	})
	if checksum != 27497 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_get_by_text: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_get_default_timeout()
	})
	if checksum != 57791 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_get_default_timeout: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_go_back()
	})
	if checksum != 60849 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_go_back: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_go_forward()
	})
	if checksum != 725 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_go_forward: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_handle_js_dialog()
	})
	if checksum != 6178 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_handle_js_dialog: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_inject_file()
	})
	if checksum != 16195 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_inject_file: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled()
	})
	if checksum != 59945 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_keyboard_down()
	})
	if checksum != 53133 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_keyboard_down: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_keyboard_press()
	})
	if checksum != 43593 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_keyboard_press: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_keyboard_type()
	})
	if checksum != 53179 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_keyboard_type: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_keyboard_up()
	})
	if checksum != 24408 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_keyboard_up: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_listens_to()
	})
	if checksum != 42886 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_listens_to: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_main_frame()
	})
	if checksum != 10285 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_main_frame: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_metrics()
	})
	if checksum != 16960 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_metrics: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_mouse_down()
	})
	if checksum != 52368 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_mouse_down: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_mouse_up()
	})
	if checksum != 52299 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_mouse_up: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_move_mouse()
	})
	if checksum != 17586 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_move_mouse: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_navigate()
	})
	if checksum != 51495 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_navigate: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_on()
	})
	if checksum != 15602 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_on: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_once()
	})
	if checksum != 33401 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_once: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_pdf()
	})
	if checksum != 50825 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_pdf: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_press()
	})
	if checksum != 21741 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_press: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_query_selector_all()
	})
	if checksum != 64158 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_query_selector_all: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_query_selector_xpath()
	})
	if checksum != 48442 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_query_selector_xpath: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_raw_window_bounds()
	})
	if checksum != 13012 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_raw_window_bounds: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_reload()
	})
	if checksum != 20867 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_reload: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_remove_all_listeners()
	})
	if checksum != 31887 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_remove_all_listeners: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_remove_listener()
	})
	if checksum != 35484 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_remove_listener: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_remove_script()
	})
	if checksum != 13802 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_remove_script: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_request()
	})
	if checksum != 58954 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_request: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_requests()
	})
	if checksum != 29432 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_requests: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_route()
	})
	if checksum != 24223 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_route: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_route_abort()
	})
	if checksum != 32588 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_route_abort: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_route_from_har()
	})
	if checksum != 9232 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_route_from_har: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_route_fulfill()
	})
	if checksum != 29300 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_route_fulfill: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_screenshot()
	})
	if checksum != 62867 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_screenshot: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_screenshot_full()
	})
	if checksum != 56180 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_screenshot_full: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_select_option()
	})
	if checksum != 5310 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_select_option: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_cache_enabled()
	})
	if checksum != 36286 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_cache_enabled: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_content()
	})
	if checksum != 60133 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_content: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_default_timeout()
	})
	if checksum != 58523 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_default_timeout: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_drag_interception()
	})
	if checksum != 35102 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_drag_interception: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_emulated_media_features()
	})
	if checksum != 49723 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_emulated_media_features: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_extra_http_headers()
	})
	if checksum != 43902 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_extra_http_headers: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_input_files()
	})
	if checksum != 1582 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_input_files: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_javascript_enabled()
	})
	if checksum != 56309 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_javascript_enabled: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_offline()
	})
	if checksum != 37394 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_offline: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_request_interception()
	})
	if checksum != 47016 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_request_interception: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_storage_state()
	})
	if checksum != 55671 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_storage_state: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_user_agent()
	})
	if checksum != 65506 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_user_agent: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_viewport_size()
	})
	if checksum != 47964 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_viewport_size: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_window_bounds()
	})
	if checksum != 34825 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_window_bounds: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_window_position()
	})
	if checksum != 55844 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_window_position: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_window_size()
	})
	if checksum != 39920 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_window_size: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_set_window_state()
	})
	if checksum != 29849 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_set_window_state: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_start_screencast()
	})
	if checksum != 3179 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_start_screencast: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_start_tracing()
	})
	if checksum != 44969 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_start_tracing: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_stop_screencast()
	})
	if checksum != 14965 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_stop_screencast: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_stop_tracing()
	})
	if checksum != 507 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_stop_tracing: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_storage_state()
	})
	if checksum != 8033 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_storage_state: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_target_id()
	})
	if checksum != 16602 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_target_id: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_title()
	})
	if checksum != 57758 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_title: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_touch_tap()
	})
	if checksum != 50285 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_touch_tap: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_unroute()
	})
	if checksum != 49265 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_unroute: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_unroute_all()
	})
	if checksum != 21098 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_unroute_all: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_url()
	})
	if checksum != 13992 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_url: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_event()
	})
	if checksum != 16279 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_event: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_event_default()
	})
	if checksum != 30417 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_event_default: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_function()
	})
	if checksum != 39925 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_function: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_navigation()
	})
	if checksum != 28813 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_navigation: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_selector()
	})
	if checksum != 58306 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_selector: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_wait_for_xpath()
	})
	if checksum != 14726 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_wait_for_xpath: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_window_id()
	})
	if checksum != 52015 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_window_id: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_window_position()
	})
	if checksum != 23821 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_window_position: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_window_rect()
	})
	if checksum != 55898 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_window_rect: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_page_window_size()
	})
	if checksum != 35222 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_page_window_size: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_pluginhandle_invoke()
	})
	if checksum != 29371 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_pluginhandle_invoke: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_pluginhandle_ops()
	})
	if checksum != 11713 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_pluginhandle_ops: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_method_pluginhandle_plugin_name()
	})
	if checksum != 61258 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_method_pluginhandle_plugin_name: UniFFI API checksum mismatch")
	}
	}
	{
	checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
		return C.uniffi_xcelerate_checksum_constructor_browser_launch()
	})
	if checksum != 45323 {
		// If this happens try cleaning and rebuilding your project
		panic("xcelerate: uniffi_xcelerate_checksum_constructor_browser_launch: UniFFI API checksum mismatch")
	}
	}
}



type FfiConverterUint64 struct{}

var FfiConverterUint64INSTANCE = FfiConverterUint64{}

func (FfiConverterUint64) Lower(value uint64) C.uint64_t {
	return C.uint64_t(value)
}

func (FfiConverterUint64) Write(writer io.Writer, value uint64) {
	writeUint64(writer, value)
}

func (FfiConverterUint64) Lift(value C.uint64_t) uint64 {
	return uint64(value)
}

func (FfiConverterUint64) Read(reader io.Reader) uint64 {
	return readUint64(reader)
}

type FfiDestroyerUint64 struct {}

func (FfiDestroyerUint64) Destroy(_ uint64) {}

type FfiConverterInt64 struct{}

var FfiConverterInt64INSTANCE = FfiConverterInt64{}

func (FfiConverterInt64) Lower(value int64) C.int64_t {
	return C.int64_t(value)
}

func (FfiConverterInt64) Write(writer io.Writer, value int64) {
	writeInt64(writer, value)
}

func (FfiConverterInt64) Lift(value C.int64_t) int64 {
	return int64(value)
}

func (FfiConverterInt64) Read(reader io.Reader) int64 {
	return readInt64(reader)
}

type FfiDestroyerInt64 struct {}

func (FfiDestroyerInt64) Destroy(_ int64) {}

type FfiConverterFloat64 struct{}

var FfiConverterFloat64INSTANCE = FfiConverterFloat64{}

func (FfiConverterFloat64) Lower(value float64) C.double {
	return C.double(value)
}

func (FfiConverterFloat64) Write(writer io.Writer, value float64) {
	writeFloat64(writer, value)
}

func (FfiConverterFloat64) Lift(value C.double) float64 {
	return float64(value)
}

func (FfiConverterFloat64) Read(reader io.Reader) float64 {
	return readFloat64(reader)
}

type FfiDestroyerFloat64 struct {}

func (FfiDestroyerFloat64) Destroy(_ float64) {}

type FfiConverterBool struct{}

var FfiConverterBoolINSTANCE = FfiConverterBool{}

func (FfiConverterBool) Lower(value bool) C.int8_t {
	if value {
		return C.int8_t(1)
	}
	return C.int8_t(0)
}

func (FfiConverterBool) Write(writer io.Writer, value bool) {
	if value {
		writeInt8(writer, 1)
	} else {
		writeInt8(writer, 0)
	}
}

func (FfiConverterBool) Lift(value C.int8_t) bool {
	return value != 0
}

func (FfiConverterBool) Read(reader io.Reader) bool {
	return readInt8(reader) != 0
}

type FfiDestroyerBool struct {}

func (FfiDestroyerBool) Destroy(_ bool) {}

type FfiConverterString struct{}

var FfiConverterStringINSTANCE = FfiConverterString{}

func (FfiConverterString) Lift(rb RustBufferI) string {
	defer rb.Free()
	reader := rb.AsReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return string(b)
}

func (FfiConverterString) Read(reader io.Reader) string {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil && err != io.EOF {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading string, expected %d, read %d", length, read_length))
	}
	return string(buffer)
}

func (FfiConverterString) Lower(value string) C.RustBuffer {
	return stringToRustBuffer(value)
}

func (c FfiConverterString) LowerExternal(value string) ExternalCRustBuffer {
	return RustBufferFromC(stringToRustBuffer(value))
}

func (FfiConverterString) Write(writer io.Writer, value string) {
	if len(value) > math.MaxInt32 {
		panic("String is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := io.WriteString(writer, value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing string, expected %d, written %d", len(value), write_length))
	}
}

type FfiDestroyerString struct {}

func (FfiDestroyerString) Destroy(_ string) {}

type FfiConverterBytes struct{}

var FfiConverterBytesINSTANCE = FfiConverterBytes{}

func (c FfiConverterBytes) Lower(value []byte) C.RustBuffer {
	return LowerIntoRustBuffer[[]byte](c, value)
}

func (c FfiConverterBytes) LowerExternal(value []byte) ExternalCRustBuffer {
	return RustBufferFromC(c.Lower(value))
}

func (c FfiConverterBytes) Write(writer io.Writer, value []byte) {
	if len(value) > math.MaxInt32 {
		panic("[]byte is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := writer.Write(value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing []byte, expected %d, written %d", len(value), write_length))
	}
}

func (c FfiConverterBytes) Lift(rb RustBufferI) []byte {
	return LiftFromRustBuffer[[]byte](c, rb)
}

func (c FfiConverterBytes) Read(reader io.Reader) []byte {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil && err != io.EOF {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading []byte, expected %d, read %d", length, read_length))
	}
	return buffer
}

type FfiDestroyerBytes struct {}

func (FfiDestroyerBytes) Destroy(_ []byte) {}



// Below is an implementation of synchronization requirements outlined in the link.
// https://github.com/mozilla/uniffi-rs/blob/0dc031132d9493ca812c3af6e7dd60ad2ea95bf0/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt#L31

type FfiObject struct {
	handle C.uint64_t
	callCounter atomic.Int64
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t
	freeFunction func(C.uint64_t, *C.RustCallStatus)
	destroyed atomic.Bool
}

func newFfiObject(
	handle C.uint64_t,
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t,
	freeFunction func(C.uint64_t, *C.RustCallStatus),
) FfiObject {
	return FfiObject {
		handle: handle,
		cloneFunction: cloneFunction,
		freeFunction: freeFunction,
	}
}

func (ffiObject *FfiObject)incrementPointer(debugName string) C.uint64_t {
	for {
		counter := ffiObject.callCounter.Load()
		if counter <= -1 {
			panic(fmt.Errorf("%v object has already been destroyed", debugName))
		}
		if counter == math.MaxInt64 {
			panic(fmt.Errorf("%v object call counter would overflow", debugName))
		}
		if ffiObject.callCounter.CompareAndSwap(counter, counter + 1) {
			break
		}
	}

	return rustCall(func(status *C.RustCallStatus) C.uint64_t {
		return ffiObject.cloneFunction(ffiObject.handle, status)
	})
}

func (ffiObject *FfiObject)decrementPointer() {
	if ffiObject.callCounter.Add(-1) == -1 {
		ffiObject.freeRustArcPtr()
	}
}

func (ffiObject *FfiObject)destroy() {
	if ffiObject.destroyed.CompareAndSwap(false, true) {
		if ffiObject.callCounter.Add(-1) == -1 {
			ffiObject.freeRustArcPtr()
		}
	}
}

func (ffiObject *FfiObject)freeRustArcPtr() {
	if ffiObject.handle == 0 {
		return
	}
	rustCall(func(status *C.RustCallStatus) int32 {
		ffiObject.freeFunction(ffiObject.handle, status)
		return 0
	})
}
// Represents a browser instance (e.g., Chrome or Edge).
type BrowserInterface interface {
	// Returns the plugin audit log as a JSON array (no secrets are recorded).
	AuditLog() string
	// Verifies the integrity of the append-only plugin audit log.
	AuditVerify() bool
	// Names of all compiled-in first-party plugins (the catalog).
	AvailablePlugins() []string
	// Returns the browser context ids as a JSON array.
	BrowserContexts() (string, error)
	// Returns the browser version info as JSON.
	Capabilities() (string, error)
	// Closes the browser and kills the process.
	Close() error
	// Returns all browser cookies as a JSON array.
	Cookies() (string, error)
	// Deletes cookies with the given name.
	DeleteCookie(name string) error
	// Returns the registered event names.
	EventNames() []string
	// Grants permissions (JSON array) to an origin.
	GrantPermissions(origin string, permissionsJson string) error
	// Whether the underlying connection is alive.
	IsConnected() bool
	// Whether an event name is registered.
	ListensTo(eventName string) bool
	// Loads a third-party plugin. Not supported in this phase.
	//
	// The sandboxed, out-of-process runner required for untrusted plugins does
	// not exist yet, so this always refuses rather than executing unknown code.
	LoadPlugin(path string) (string, error)
	// Creates a new (incognito) browser context and returns its id.
	NewContext() (string, error)
	NewPage(url string) (*Page, error)
	// Registers interest in a root-session CDP event.
	On(eventName string) 
	// Alias for [`Browser::on`].
	Once(eventName string) 
	// Returns a handle to an enabled plugin so its ops can be invoked.
	Plugin(name string) (*PluginHandle, error)
	// Names of the plugins currently enabled on this browser.
	PluginNames() []string
	// Removes every registered listener.
	RemoveAllListeners() 
	// Removes a single registered listener.
	RemoveListener(eventName string) 
	// Resets all permission overrides.
	ResetPermissions() error
	// Sets a cookie from a JSON object.
	SetCookie(cookieJson string) error
	// Sets the download directory for the browser.
	SetDownloadBehavior(path string) error
	// Starts CDP tracing.
	StartTracing() error
	// Stops CDP tracing.
	StopTracing() error
	// Returns the current targets as a JSON array (`Target.getTargets`).
	Targets() (string, error)
	// Enables a compiled-in first-party plugin at runtime.
	//
	// Launch-time contributions (such as binary patching) only take effect if
	// the plugin was enabled before the browser launched; enabling a plugin
	// afterwards applies its runtime hooks to pages created from now on. This
	// is audited as a runtime enable. Unknown or third-party names are refused.
	UsePlugin(name string) error
	// Returns the browser's user agent.
	UserAgent() (string, error)
	// Returns the browser version information.
	Version() (string, error)
	// Waits for the next root-session CDP event named `event_name`.
	WaitForEvent(eventName string, timeoutMs uint64) (string, error)
	// [`Browser::wait_for_event`] with the default 30s timeout.
	WaitForEventDefault(eventName string) (string, error)
	// The WebSocket endpoint Chrome was launched with.
	WsEndpoint() string
}
// Represents a browser instance (e.g., Chrome or Edge).
type Browser struct {
	ffiObject FfiObject
}


func BrowserLaunch(config BrowserConfig) (*Browser, error) {
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Browser {
			return FfiConverterBrowserINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_constructor_browser_launch(FfiConverterBrowserConfigINSTANCE.Lower(config)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}



// Returns the plugin audit log as a JSON array (no secrets are recorded).
func (_self *Browser) AuditLog() string {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_browser_audit_log(
		_pointer,_uniffiStatus),
	}
	}))
}

// Verifies the integrity of the append-only plugin audit log.
func (_self *Browser) AuditVerify() bool {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterBoolINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.int8_t {
		return C.uniffi_xcelerate_fn_method_browser_audit_verify(
		_pointer,_uniffiStatus)
	}))
}

// Names of all compiled-in first-party plugins (the catalog).
func (_self *Browser) AvailablePlugins() []string {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequenceStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_browser_available_plugins(
		_pointer,_uniffiStatus),
	}
	}))
}

// Returns the browser context ids as a JSON array.
func (_self *Browser) BrowserContexts() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_browser_contexts(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the browser version info as JSON.
func (_self *Browser) Capabilities() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_capabilities(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Closes the browser and kills the process.
func (_self *Browser) Close() error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_close(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns all browser cookies as a JSON array.
func (_self *Browser) Cookies() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_cookies(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Deletes cookies with the given name.
func (_self *Browser) DeleteCookie(name string) error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_delete_cookie(
		_pointer,FfiConverterStringINSTANCE.Lower(name)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the registered event names.
func (_self *Browser) EventNames() []string {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []string {
			return FfiConverterSequenceStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_event_names(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	

	return res 
}

// Grants permissions (JSON array) to an origin.
func (_self *Browser) GrantPermissions(origin string, permissionsJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_grant_permissions(
		_pointer,FfiConverterStringINSTANCE.Lower(origin), FfiConverterStringINSTANCE.Lower(permissionsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Whether the underlying connection is alive.
func (_self *Browser) IsConnected() bool {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_is_connected(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	

	return res 
}

// Whether an event name is registered.
func (_self *Browser) ListensTo(eventName string) bool {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_listens_to(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	

	return res 
}

// Loads a third-party plugin. Not supported in this phase.
//
// The sandboxed, out-of-process runner required for untrusted plugins does
// not exist yet, so this always refuses rather than executing unknown code.
func (_self *Browser) LoadPlugin(path string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*XcelerateError](FfiConverterXcelerateError{},func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_browser_load_plugin(
		_pointer,FfiConverterStringINSTANCE.Lower(path),_uniffiStatus),
	}
	})
		if _uniffiErr != nil {
			var _uniffiDefaultValue string
			return _uniffiDefaultValue, _uniffiErr
		} else {
			return FfiConverterStringINSTANCE.Lift(_uniffiRV), nil
		}
}

// Creates a new (incognito) browser context and returns its id.
func (_self *Browser) NewContext() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_new_context(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Browser) NewPage(url string) (*Page, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Page {
			return FfiConverterPageINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_new_page(
		_pointer,FfiConverterStringINSTANCE.Lower(url)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Registers interest in a root-session CDP event.
func (_self *Browser) On(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_on(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Alias for [`Browser::on`].
func (_self *Browser) Once(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_once(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Returns a handle to an enabled plugin so its ops can be invoked.
func (_self *Browser) Plugin(name string) (*PluginHandle, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*XcelerateError](FfiConverterXcelerateError{},func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_xcelerate_fn_method_browser_plugin(
		_pointer,FfiConverterStringINSTANCE.Lower(name),_uniffiStatus)
	})
		if _uniffiErr != nil {
			var _uniffiDefaultValue *PluginHandle
			return _uniffiDefaultValue, _uniffiErr
		} else {
			return FfiConverterPluginHandleINSTANCE.Lift(_uniffiRV), nil
		}
}

// Names of the plugins currently enabled on this browser.
func (_self *Browser) PluginNames() []string {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequenceStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_browser_plugin_names(
		_pointer,_uniffiStatus),
	}
	}))
}

// Removes every registered listener.
func (_self *Browser) RemoveAllListeners()  {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_remove_all_listeners(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Removes a single registered listener.
func (_self *Browser) RemoveListener(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_remove_listener(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Resets all permission overrides.
func (_self *Browser) ResetPermissions() error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_reset_permissions(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets a cookie from a JSON object.
func (_self *Browser) SetCookie(cookieJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_set_cookie(
		_pointer,FfiConverterStringINSTANCE.Lower(cookieJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets the download directory for the browser.
func (_self *Browser) SetDownloadBehavior(path string) error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_set_download_behavior(
		_pointer,FfiConverterStringINSTANCE.Lower(path)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Starts CDP tracing.
func (_self *Browser) StartTracing() error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_start_tracing(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Stops CDP tracing.
func (_self *Browser) StopTracing() error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_stop_tracing(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the current targets as a JSON array (`Target.getTargets`).
func (_self *Browser) Targets() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_targets(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Enables a compiled-in first-party plugin at runtime.
//
// Launch-time contributions (such as binary patching) only take effect if
// the plugin was enabled before the browser launched; enabling a plugin
// afterwards applies its runtime hooks to pages created from now on. This
// is audited as a runtime enable. Unknown or third-party names are refused.
func (_self *Browser) UsePlugin(name string) error {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_browser_use_plugin(
		_pointer,FfiConverterStringINSTANCE.Lower(name)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the browser's user agent.
func (_self *Browser) UserAgent() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_user_agent(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the browser version information.
func (_self *Browser) Version() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_version(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Waits for the next root-session CDP event named `event_name`.
func (_self *Browser) WaitForEvent(eventName string, timeoutMs uint64) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_wait_for_event(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName), FfiConverterUint64INSTANCE.Lower(timeoutMs)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// [`Browser::wait_for_event`] with the default 30s timeout.
func (_self *Browser) WaitForEventDefault(eventName string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_browser_wait_for_event_default(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// The WebSocket endpoint Chrome was launched with.
func (_self *Browser) WsEndpoint() string {
	_pointer := _self.ffiObject.incrementPointer("*Browser")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_browser_ws_endpoint(
		_pointer,_uniffiStatus),
	}
	}))
}
func (object *Browser) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterBrowser struct {}

var FfiConverterBrowserINSTANCE = FfiConverterBrowser{}


func (c FfiConverterBrowser) Lift(handle C.uint64_t) *Browser {
	result := &Browser {
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_xcelerate_fn_clone_browser(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_xcelerate_fn_free_browser(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*Browser).Destroy)
	return result
}

func (c FfiConverterBrowser) Read(reader io.Reader) *Browser {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterBrowser) Lower(value *Browser) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*Browser")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterBrowser) Write(writer io.Writer, value *Browser) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalBrowser(handle uint64) *Browser {
	return FfiConverterBrowserINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalBrowser(value *Browser) uint64 {
	return uint64(FfiConverterBrowserINSTANCE.Lower(value))
}

type FfiDestroyerBrowser struct {}

func (_ FfiDestroyerBrowser) Destroy(value *Browser) {
		value.Destroy()
}



// Represents an HTML element in the DOM.
type ElementInterface interface {
	// Returns the value of a specific attribute.
	Attribute(name string) (*string, error)
	// Like [`Element::call_json`] but coerces the result to a bool.
	CallBool(function string, argsJson string) (bool, error)
	// Calls a JS function on this element with JSON-encoded arguments.
	CallJson(function string, argsJson string) (string, error)
	// Runs a JS function against the first descendant matching `selector`.
	CallOnSelector(selector string, expression string) (string, error)
	// Runs a JS function against every descendant matching `selector`.
	CallOnSelectorAll(selector string, expression string) (string, error)
	// Like [`Element::call_json`] but coerces the result to a string.
	CallString(function string, argsJson string) (string, error)
	// Clicks the element.
	Click() (*Element, error)
	// Clicks the element using realistic mouse movement and CDP input events.
	ClickStealth() (*Element, error)
	// Number of elements this handle represents (always 1).
	Count() (int64, error)
	// Releases the underlying remote object handle.
	Dispose() error
	// Calls a function on this element and coerces the result to a bool.
	EvaluateBool(function string) (bool, error)
	// Calls a JS function on this element and returns the resulting node.
	EvaluateHandle(function string) (*Element, error)
	// Calls a function on this element; the result is returned as JSON text.
	EvaluateJson(function string) (string, error)
	// Calls a function on this element and coerces the result to a string.
	EvaluateString(function string) (string, error)
	// Focuses the element.
	Focus() (*Element, error)
	// Finds a descendant form control by its `<label>` text.
	GetByLabel(label string) (*Element, error)
	// Finds a descendant by ARIA role.
	GetByRole(role string) (*Element, error)
	// Finds a descendant whose text contains `text`.
	GetByText(text string) (*Element, error)
	// Returns this element's enumerable properties as a JSON object.
	GetProperties() (string, error)
	// Hovers over the element.
	Hover() (*Element, error)
	// Hovers over the element using realistic mouse movement.
	HoverStealth() (*Element, error)
	// Returns the inner HTML of the element.
	InnerHtml() (string, error)
	// Focuses the element and presses a key.
	Press(key string) error
	// Returns the first descendant matching `selector` as an [`Element`].
	QuerySelector(selector string) (*Element, error)
	// Returns every descendant matching `selector`.
	//
	// Resolves the whole node list with a single `Runtime.getProperties` call
	// rather than one `evaluate` per match.
	QuerySelectorAll(selector string) ([]*Element, error)
	// Finds a descendant by attribute value.
	QuerySelectorAttr(attribute string, value string) (*Element, error)
	// Finds a descendant matching an XPath expression.
	QuerySelectorXpath(xpath string) (*Element, error)
	// Captures a PNG screenshot cropped to this element.
	Screenshot() ([]byte, error)
	// Captures a base64 PNG screenshot cropped to this element.
	ScreenshotBase64() (string, error)
	// Selects options by value or label on this `<select>` element.
	SelectOption(valuesJson string) error
	// Sets the files of this `<input type="file">` element.
	SetInputFiles(filesJson string) error
	// Returns the visible text of the element.
	Text() (string, error)
	TypeText(text string) (*Element, error)
	// Waits for a descendant matching `selector` to appear.
	WaitForSelector(selector string) (*Element, error)
}
// Represents an HTML element in the DOM.
type Element struct {
	ffiObject FfiObject
}




// Returns the value of a specific attribute.
func (_self *Element) Attribute(name string) (*string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) *string {
			return FfiConverterOptionalStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_attribute(
		_pointer,FfiConverterStringINSTANCE.Lower(name)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Like [`Element::call_json`] but coerces the result to a bool.
func (_self *Element) CallBool(function string, argsJson string) (bool, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_call_bool(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Calls a JS function on this element with JSON-encoded arguments.
func (_self *Element) CallJson(function string, argsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_call_json(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Runs a JS function against the first descendant matching `selector`.
func (_self *Element) CallOnSelector(selector string, expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_call_on_selector(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Runs a JS function against every descendant matching `selector`.
func (_self *Element) CallOnSelectorAll(selector string, expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_call_on_selector_all(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Like [`Element::call_json`] but coerces the result to a string.
func (_self *Element) CallString(function string, argsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_call_string(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Clicks the element.
func (_self *Element) Click() (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_click(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Clicks the element using realistic mouse movement and CDP input events.
func (_self *Element) ClickStealth() (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_click_stealth(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Number of elements this handle represents (always 1).
func (_self *Element) Count() (int64, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int64_t {
			res := C.ffi_xcelerate_rust_future_complete_i64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int64_t) int64 {
			return FfiConverterInt64INSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_count(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Releases the underlying remote object handle.
func (_self *Element) Dispose() error {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_element_dispose(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Calls a function on this element and coerces the result to a bool.
func (_self *Element) EvaluateBool(function string) (bool, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_evaluate_bool(
		_pointer,FfiConverterStringINSTANCE.Lower(function)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Calls a JS function on this element and returns the resulting node.
func (_self *Element) EvaluateHandle(function string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_evaluate_handle(
		_pointer,FfiConverterStringINSTANCE.Lower(function)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Calls a function on this element; the result is returned as JSON text.
func (_self *Element) EvaluateJson(function string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_evaluate_json(
		_pointer,FfiConverterStringINSTANCE.Lower(function)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Calls a function on this element and coerces the result to a string.
func (_self *Element) EvaluateString(function string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_evaluate_string(
		_pointer,FfiConverterStringINSTANCE.Lower(function)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Focuses the element.
func (_self *Element) Focus() (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_focus(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a descendant form control by its `<label>` text.
func (_self *Element) GetByLabel(label string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_get_by_label(
		_pointer,FfiConverterStringINSTANCE.Lower(label)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a descendant by ARIA role.
func (_self *Element) GetByRole(role string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_get_by_role(
		_pointer,FfiConverterStringINSTANCE.Lower(role)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a descendant whose text contains `text`.
func (_self *Element) GetByText(text string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_get_by_text(
		_pointer,FfiConverterStringINSTANCE.Lower(text)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns this element's enumerable properties as a JSON object.
func (_self *Element) GetProperties() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_get_properties(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Hovers over the element.
func (_self *Element) Hover() (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_hover(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Hovers over the element using realistic mouse movement.
func (_self *Element) HoverStealth() (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_hover_stealth(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the inner HTML of the element.
func (_self *Element) InnerHtml() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_inner_html(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Focuses the element and presses a key.
func (_self *Element) Press(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_element_press(
		_pointer,FfiConverterStringINSTANCE.Lower(key)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the first descendant matching `selector` as an [`Element`].
func (_self *Element) QuerySelector(selector string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_query_selector(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns every descendant matching `selector`.
//
// Resolves the whole node list with a single `Runtime.getProperties` call
// rather than one `evaluate` per match.
func (_self *Element) QuerySelectorAll(selector string) ([]*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []*Element {
			return FfiConverterSequenceElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_query_selector_all(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a descendant by attribute value.
func (_self *Element) QuerySelectorAttr(attribute string, value string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_query_selector_attr(
		_pointer,FfiConverterStringINSTANCE.Lower(attribute), FfiConverterStringINSTANCE.Lower(value)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a descendant matching an XPath expression.
func (_self *Element) QuerySelectorXpath(xpath string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_query_selector_xpath(
		_pointer,FfiConverterStringINSTANCE.Lower(xpath)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Captures a PNG screenshot cropped to this element.
func (_self *Element) Screenshot() ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []byte {
			return FfiConverterBytesINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_screenshot(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Captures a base64 PNG screenshot cropped to this element.
func (_self *Element) ScreenshotBase64() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_screenshot_base64(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Selects options by value or label on this `<select>` element.
func (_self *Element) SelectOption(valuesJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_element_select_option(
		_pointer,FfiConverterStringINSTANCE.Lower(valuesJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets the files of this `<input type="file">` element.
func (_self *Element) SetInputFiles(filesJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_element_set_input_files(
		_pointer,FfiConverterStringINSTANCE.Lower(filesJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the visible text of the element.
func (_self *Element) Text() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_text(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Element) TypeText(text string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_type_text(
		_pointer,FfiConverterStringINSTANCE.Lower(text)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Waits for a descendant matching `selector` to appear.
func (_self *Element) WaitForSelector(selector string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Element")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_element_wait_for_selector(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}
func (object *Element) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterElement struct {}

var FfiConverterElementINSTANCE = FfiConverterElement{}


func (c FfiConverterElement) Lift(handle C.uint64_t) *Element {
	result := &Element {
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_xcelerate_fn_clone_element(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_xcelerate_fn_free_element(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*Element).Destroy)
	return result
}

func (c FfiConverterElement) Read(reader io.Reader) *Element {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterElement) Lower(value *Element) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*Element")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterElement) Write(writer io.Writer, value *Element) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalElement(handle uint64) *Element {
	return FfiConverterElementINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalElement(value *Element) uint64 {
	return uint64(FfiConverterElementINSTANCE.Lower(value))
}

type FfiDestroyerElement struct {}

func (_ FfiDestroyerElement) Destroy(value *Element) {
		value.Destroy()
}



type PageInterface interface {
	// Activates this page's target.
	Activate() error
	// Activates the given target (window/tab).
	ActivateTarget(targetId string) error
	// Evaluates a script on every new document.
	AddScriptToEvaluateOnNewDocument(source string) (string, error)
	// Injects a `<style>` element and returns the injected content.
	AddStyleTag(content string) (string, error)
	// Sets the credentials used to answer HTTP auth challenges.
	Authenticate(username string, password string) error
	// Brings the page to the front.
	BringToFront() error
	// Like [`Page::call_json`] but coerces the result to a bool.
	CallBool(function string, argsJson string) (bool, error)
	// Calls a JS function with JSON-encoded arguments, returning JSON text.
	//
	// This is the shim the adapters use to express the broad upstream surface
	// as data (a function body per method) rather than a core method per member.
	CallJson(function string, argsJson string) (string, error)
	// Runs a JS function against the element matching `selector` (`$eval`).
	CallOnSelector(selector string, expression string) (string, error)
	// Runs a JS function against every element matching `selector` (`$$eval`).
	CallOnSelectorAll(selector string, expression string) (string, error)
	// Like [`Page::call_json`] but coerces the result to a string.
	CallString(function string, argsJson string) (string, error)
	// Clears the recorded intercepted requests.
	ClearRequests() 
	// Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
	ClickMouse(x float64, y float64) (*Page, error)
	// Closes the page.
	Close() error
	// Returns the full HTML content of the page.
	Content() (string, error)
	// Returns a single cookie by name as JSON (or null).
	Cookie(name string) (string, error)
	// Returns the cookies visible to this page as a JSON array.
	Cookies() (string, error)
	// Starts CSS coverage collection.
	CoverageStartCss() error
	// Starts JS coverage collection.
	CoverageStartJs() error
	// Stops CSS coverage collection and returns the result as JSON.
	CoverageStopCss() (string, error)
	// Stops JS coverage collection and returns the result as JSON.
	CoverageStopJs() (string, error)
	// Returns the page PDF as a base64 string.
	CreatePdfStream() (string, error)
	DecodeBase64(data string) ([]byte, error)
	// Overrides the idle state.
	EmulateIdleState(isUserActive bool, isScreenUnlocked bool) error
	// Emulates a media type and/or colour scheme.
	EmulateMedia(media *string, colorScheme *string) error
	EnsureInterception() 
	// Evaluates JavaScript and coerces the result to a bool.
	EvaluateBool(expression string) (bool, error)
	// Evaluates JavaScript and returns the resulting object as an [`Element`].
	EvaluateHandle(expression string) (*Element, error)
	// Evaluates JavaScript in the page and returns the result as a JSON string.
	EvaluateJson(expression string) (string, error)
	// Evaluates JavaScript and coerces the result to a string.
	EvaluateString(expression string) (string, error)
	// Returns the registered event names.
	EventNames() []string
	// Escape hatch: sends an arbitrary CDP command and returns its JSON result.
	//
	// The adapters use this to express CDP-backed library methods as data
	// (a method name + parameter template), keeping the core surface small.
	ExecuteCdpCmd(method string, paramsJson string) (string, error)
	// Finds an element matching the CSS selector.
	FindElement(selector string) (*Element, error)
	// Returns the frame matching an id or name as JSON (or null).
	Frame(frameId string) (string, error)
	// Returns the main frame's name.
	FrameName() (string, error)
	// Returns every frame in the page as a JSON array.
	Frames() (string, error)
	// Finds a form control by its associated `<label>` text.
	GetByLabel(label string) (*Element, error)
	// Finds an element by ARIA role (falls back to a tag-name lookup).
	GetByRole(role string) (*Element, error)
	// Finds an element whose text content contains `text`.
	GetByText(text string) (*Element, error)
	// Returns the stored default timeout (ms).
	GetDefaultTimeout() (float64, error)
	GoBack() error
	// Navigates forward in history.
	GoForward() error
	// Accepts or dismisses the active JavaScript dialog.
	HandleJsDialog(accept bool, promptText *string) error
	// Reads a local file and injects it as an init script.
	InjectFile(path string) (string, error)
	// Whether drag interception is enabled.
	IsDragInterceptionEnabled() bool
	// Dispatches a keydown event for `key`.
	KeyboardDown(key string) error
	// Presses `key` on the focused element.
	KeyboardPress(key string) error
	// Types `text` into the focused element.
	KeyboardType(text string) error
	// Dispatches a keyup event for `key`.
	KeyboardUp(key string) error
	// Whether an event name is registered.
	ListensTo(eventName string) bool
	// Returns the main frame as JSON.
	MainFrame() (string, error)
	// Returns the page performance metrics as a JSON object.
	Metrics() (string, error)
	// Triggers a mousePress event at the current mouse coordinates.
	MouseDown(button string) (*Page, error)
	// Triggers a mouseReleased event at the current mouse coordinates.
	MouseUp(button string) (*Page, error)
	// Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
	MoveMouse(x float64, y float64) (*Page, error)
	// Navigates to a URL.
	Navigate(url string) error
	// Registers interest in a CDP event name.
	On(eventName string) 
	// Alias for [`Page::on`].
	Once(eventName string) 
	Pdf() ([]byte, error)
	// Focuses the element matching `selector` and presses `key`.
	Press(selector string, key string) error
	// Returns every element matching the CSS selector.
	//
	// Uses two round trips (fetch the node list, then read its properties)
	// instead of one `evaluate` per match.
	QuerySelectorAll(selector string) ([]*Element, error)
	// Returns the first node matching an XPath expression as an [`Element`].
	QuerySelectorXpath(xpath string) (*Element, error)
	RawWindowBounds() (string, error)
	// Reloads the page.
	Reload() error
	// Removes every registered event listener.
	RemoveAllListeners() 
	// Removes a single registered event listener.
	RemoveListener(eventName string) 
	// Removes an init script by identifier.
	RemoveScript(identifier string) error
	// Returns the most recent intercepted request as JSON.
	Request() (string, error)
	// Returns the intercepted requests seen so far as JSON.
	Requests() (string, error)
	// Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
	Route(pattern string, action string, body *string, contentType *string) error
	// Aborts every request matching `pattern`.
	RouteAbort(pattern string) error
	// Registers fulfill routes for every entry in a HAR file.
	RouteFromHar(path string) error
	// Fulfills every request matching `pattern` with `body`.
	RouteFulfill(pattern string, body string, contentType *string) error
	Screenshot() ([]byte, error)
	ScreenshotFull() ([]byte, error)
	// Selects options by value/label on the matching `<select>`.
	SelectOption(selector string, valuesJson string) error
	// Enables or disables the HTTP cache.
	SetCacheEnabled(enabled bool) error
	// Replaces the document content.
	SetContent(html string) error
	// Stores a default timeout (ms) for adapter compatibility.
	SetDefaultTimeout(milliseconds float64) error
	// Enables or disables input drag interception.
	SetDragInterception(enabled bool) error
	// Overrides media features (JSON array of `{name,value}`).
	SetEmulatedMediaFeatures(featuresJson string) error
	// Sets extra HTTP headers for every request from this page.
	SetExtraHttpHeaders(headersJson string) error
	// Sets the files of the matching `<input type="file">`.
	SetInputFiles(selector string, filesJson string) error
	// Enables or disables JavaScript execution.
	SetJavascriptEnabled(enabled bool) error
	// Toggles offline mode.
	SetOffline(offline bool) error
	// Enables or disables request interception for this page.
	SetRequestInterception(enabled bool) error
	// Restores cookies + localStorage from a storage-state JSON object.
	SetStorageState(stateJson string) error
	// Overrides `navigator.userAgent` for this page.
	SetUserAgent(userAgent string, acceptLanguage *string) error
	// Overrides the viewport size.
	SetViewportSize(width uint64, height int64) error
	// Moves and resizes the window.
	SetWindowBounds(left int64, top int64, width int64, height int64) error
	// Moves the window, preserving its size.
	SetWindowPosition(x int64, y int64) error
	// Resizes the window, preserving its position.
	SetWindowSize(width int64, height int64) error
	// Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
	SetWindowState(state string) error
	// Starts a PNG screencast.
	StartScreencast() error
	// Starts CDP tracing on this page's session.
	StartTracing() error
	// Stops the screencast.
	StopScreencast() error
	// Stops CDP tracing on this page's session.
	StopTracing() error
	// Returns cookies + localStorage as a storage-state JSON object.
	StorageState() (string, error)
	// The CDP target id backing this page.
	TargetId() string
	// Returns the page title.
	Title() (string, error)
	// Dispatches a touch tap at (x, y).
	TouchTap(x float64, y float64) error
	// Removes the routes registered for `pattern`.
	Unroute(pattern string) error
	// Removes every route rule.
	UnrouteAll() error
	// Returns the current document URL.
	Url() (string, error)
	// Waits for the next CDP event named `event_name` and returns its params.
	//
	// The relevant domain is enabled first (best effort), so callers do not
	// have to.
	WaitForEvent(eventName string, timeoutMs uint64) (string, error)
	// [`Page::wait_for_event`] with the default 30s timeout.
	WaitForEventDefault(eventName string) (string, error)
	// Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
	WaitForFunction(expression string, timeoutMs uint64) error
	// Waits for the page to finish loading.
	WaitForNavigation() error
	// Waits for an element matching the selector to appear in the DOM.
	WaitForSelector(selector string) (*Element, error)
	// Waits for the first XPath match to appear.
	WaitForXpath(xpath string, timeoutMs uint64) (*Element, error)
	WindowId() (int64, error)
	// Returns the window position as JSON (`{x,y}`).
	WindowPosition() (string, error)
	// Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
	WindowRect() (string, error)
	// Returns the window size as JSON (`{width,height}`).
	WindowSize() (string, error)
}
type Page struct {
	ffiObject FfiObject
}




// Activates this page's target.
func (_self *Page) Activate() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_activate(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Activates the given target (window/tab).
func (_self *Page) ActivateTarget(targetId string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_activate_target(
		_pointer,FfiConverterStringINSTANCE.Lower(targetId)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Evaluates a script on every new document.
func (_self *Page) AddScriptToEvaluateOnNewDocument(source string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(
		_pointer,FfiConverterStringINSTANCE.Lower(source)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Injects a `<style>` element and returns the injected content.
func (_self *Page) AddStyleTag(content string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_add_style_tag(
		_pointer,FfiConverterStringINSTANCE.Lower(content)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Sets the credentials used to answer HTTP auth challenges.
func (_self *Page) Authenticate(username string, password string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_authenticate(
		_pointer,FfiConverterStringINSTANCE.Lower(username), FfiConverterStringINSTANCE.Lower(password)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Brings the page to the front.
func (_self *Page) BringToFront() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_bring_to_front(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Like [`Page::call_json`] but coerces the result to a bool.
func (_self *Page) CallBool(function string, argsJson string) (bool, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_call_bool(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Calls a JS function with JSON-encoded arguments, returning JSON text.
//
// This is the shim the adapters use to express the broad upstream surface
// as data (a function body per method) rather than a core method per member.
func (_self *Page) CallJson(function string, argsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_call_json(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Runs a JS function against the element matching `selector` (`$eval`).
func (_self *Page) CallOnSelector(selector string, expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_call_on_selector(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Runs a JS function against every element matching `selector` (`$$eval`).
func (_self *Page) CallOnSelectorAll(selector string, expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_call_on_selector_all(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Like [`Page::call_json`] but coerces the result to a string.
func (_self *Page) CallString(function string, argsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_call_string(
		_pointer,FfiConverterStringINSTANCE.Lower(function), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Clears the recorded intercepted requests.
func (_self *Page) ClearRequests()  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_clear_requests(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
func (_self *Page) ClickMouse(x float64, y float64) (*Page, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Page {
			return FfiConverterPageINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_click_mouse(
		_pointer,FfiConverterFloat64INSTANCE.Lower(x), FfiConverterFloat64INSTANCE.Lower(y)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Closes the page.
func (_self *Page) Close() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_close(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the full HTML content of the page.
func (_self *Page) Content() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_content(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns a single cookie by name as JSON (or null).
func (_self *Page) Cookie(name string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_cookie(
		_pointer,FfiConverterStringINSTANCE.Lower(name)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the cookies visible to this page as a JSON array.
func (_self *Page) Cookies() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_cookies(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Starts CSS coverage collection.
func (_self *Page) CoverageStartCss() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_coverage_start_css(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Starts JS coverage collection.
func (_self *Page) CoverageStartJs() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_coverage_start_js(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Stops CSS coverage collection and returns the result as JSON.
func (_self *Page) CoverageStopCss() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_coverage_stop_css(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Stops JS coverage collection and returns the result as JSON.
func (_self *Page) CoverageStopJs() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_coverage_stop_js(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the page PDF as a base64 string.
func (_self *Page) CreatePdfStream() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_create_pdf_stream(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Page) DecodeBase64(data string) ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*XcelerateError](FfiConverterXcelerateError{},func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_page_decode_base64(
		_pointer,FfiConverterStringINSTANCE.Lower(data),_uniffiStatus),
	}
	})
		if _uniffiErr != nil {
			var _uniffiDefaultValue []byte
			return _uniffiDefaultValue, _uniffiErr
		} else {
			return FfiConverterBytesINSTANCE.Lift(_uniffiRV), nil
		}
}

// Overrides the idle state.
func (_self *Page) EmulateIdleState(isUserActive bool, isScreenUnlocked bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_emulate_idle_state(
		_pointer,FfiConverterBoolINSTANCE.Lower(isUserActive), FfiConverterBoolINSTANCE.Lower(isScreenUnlocked)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Emulates a media type and/or colour scheme.
func (_self *Page) EmulateMedia(media *string, colorScheme *string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_emulate_media(
		_pointer,FfiConverterOptionalStringINSTANCE.Lower(media), FfiConverterOptionalStringINSTANCE.Lower(colorScheme)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

func (_self *Page) EnsureInterception()  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_ensure_interception(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Evaluates JavaScript and coerces the result to a bool.
func (_self *Page) EvaluateBool(expression string) (bool, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_evaluate_bool(
		_pointer,FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Evaluates JavaScript and returns the resulting object as an [`Element`].
func (_self *Page) EvaluateHandle(expression string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_evaluate_handle(
		_pointer,FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Evaluates JavaScript in the page and returns the result as a JSON string.
func (_self *Page) EvaluateJson(expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_evaluate_json(
		_pointer,FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Evaluates JavaScript and coerces the result to a string.
func (_self *Page) EvaluateString(expression string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_evaluate_string(
		_pointer,FfiConverterStringINSTANCE.Lower(expression)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the registered event names.
func (_self *Page) EventNames() []string {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []string {
			return FfiConverterSequenceStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_event_names(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	

	return res 
}

// Escape hatch: sends an arbitrary CDP command and returns its JSON result.
//
// The adapters use this to express CDP-backed library methods as data
// (a method name + parameter template), keeping the core surface small.
func (_self *Page) ExecuteCdpCmd(method string, paramsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_execute_cdp_cmd(
		_pointer,FfiConverterStringINSTANCE.Lower(method), FfiConverterStringINSTANCE.Lower(paramsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds an element matching the CSS selector.
func (_self *Page) FindElement(selector string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_find_element(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the frame matching an id or name as JSON (or null).
func (_self *Page) Frame(frameId string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_frame(
		_pointer,FfiConverterStringINSTANCE.Lower(frameId)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the main frame's name.
func (_self *Page) FrameName() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_frame_name(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns every frame in the page as a JSON array.
func (_self *Page) Frames() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_frames(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds a form control by its associated `<label>` text.
func (_self *Page) GetByLabel(label string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_get_by_label(
		_pointer,FfiConverterStringINSTANCE.Lower(label)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds an element by ARIA role (falls back to a tag-name lookup).
func (_self *Page) GetByRole(role string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_get_by_role(
		_pointer,FfiConverterStringINSTANCE.Lower(role)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Finds an element whose text content contains `text`.
func (_self *Page) GetByText(text string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_get_by_text(
		_pointer,FfiConverterStringINSTANCE.Lower(text)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the stored default timeout (ms).
func (_self *Page) GetDefaultTimeout() (float64, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.double {
			res := C.ffi_xcelerate_rust_future_complete_f64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.double) float64 {
			return FfiConverterFloat64INSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_get_default_timeout(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_f64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_f64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Page) GoBack() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_go_back(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Navigates forward in history.
func (_self *Page) GoForward() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_go_forward(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Accepts or dismisses the active JavaScript dialog.
func (_self *Page) HandleJsDialog(accept bool, promptText *string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_handle_js_dialog(
		_pointer,FfiConverterBoolINSTANCE.Lower(accept), FfiConverterOptionalStringINSTANCE.Lower(promptText)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Reads a local file and injects it as an init script.
func (_self *Page) InjectFile(path string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_inject_file(
		_pointer,FfiConverterStringINSTANCE.Lower(path)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Whether drag interception is enabled.
func (_self *Page) IsDragInterceptionEnabled() bool {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	

	return res 
}

// Dispatches a keydown event for `key`.
func (_self *Page) KeyboardDown(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_keyboard_down(
		_pointer,FfiConverterStringINSTANCE.Lower(key)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Presses `key` on the focused element.
func (_self *Page) KeyboardPress(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_keyboard_press(
		_pointer,FfiConverterStringINSTANCE.Lower(key)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Types `text` into the focused element.
func (_self *Page) KeyboardType(text string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_keyboard_type(
		_pointer,FfiConverterStringINSTANCE.Lower(text)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Dispatches a keyup event for `key`.
func (_self *Page) KeyboardUp(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_keyboard_up(
		_pointer,FfiConverterStringINSTANCE.Lower(key)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Whether an event name is registered.
func (_self *Page) ListensTo(eventName string) bool {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, _ :=uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int8_t {
			res := C.ffi_xcelerate_rust_future_complete_i8(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int8_t) bool {
			return FfiConverterBoolINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_listens_to(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i8(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i8(handle)
		},
	)

	

	return res 
}

// Returns the main frame as JSON.
func (_self *Page) MainFrame() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_main_frame(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the page performance metrics as a JSON object.
func (_self *Page) Metrics() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_metrics(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Triggers a mousePress event at the current mouse coordinates.
func (_self *Page) MouseDown(button string) (*Page, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Page {
			return FfiConverterPageINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_mouse_down(
		_pointer,FfiConverterStringINSTANCE.Lower(button)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Triggers a mouseReleased event at the current mouse coordinates.
func (_self *Page) MouseUp(button string) (*Page, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Page {
			return FfiConverterPageINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_mouse_up(
		_pointer,FfiConverterStringINSTANCE.Lower(button)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
func (_self *Page) MoveMouse(x float64, y float64) (*Page, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Page {
			return FfiConverterPageINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_move_mouse(
		_pointer,FfiConverterFloat64INSTANCE.Lower(x), FfiConverterFloat64INSTANCE.Lower(y)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Navigates to a URL.
func (_self *Page) Navigate(url string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_navigate(
		_pointer,FfiConverterStringINSTANCE.Lower(url)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Registers interest in a CDP event name.
func (_self *Page) On(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_on(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Alias for [`Page::on`].
func (_self *Page) Once(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_once(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

func (_self *Page) Pdf() ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []byte {
			return FfiConverterBytesINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_pdf(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Focuses the element matching `selector` and presses `key`.
func (_self *Page) Press(selector string, key string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_press(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(key)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns every element matching the CSS selector.
//
// Uses two round trips (fetch the node list, then read its properties)
// instead of one `evaluate` per match.
func (_self *Page) QuerySelectorAll(selector string) ([]*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []*Element {
			return FfiConverterSequenceElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_query_selector_all(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the first node matching an XPath expression as an [`Element`].
func (_self *Page) QuerySelectorXpath(xpath string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_query_selector_xpath(
		_pointer,FfiConverterStringINSTANCE.Lower(xpath)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Page) RawWindowBounds() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_raw_window_bounds(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Reloads the page.
func (_self *Page) Reload() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_reload(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Removes every registered event listener.
func (_self *Page) RemoveAllListeners()  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_remove_all_listeners(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Removes a single registered event listener.
func (_self *Page) RemoveListener(eventName string)  {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	uniffiRustCallAsync[error](
        nil,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_remove_listener(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	

	
}

// Removes an init script by identifier.
func (_self *Page) RemoveScript(identifier string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_remove_script(
		_pointer,FfiConverterStringINSTANCE.Lower(identifier)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the most recent intercepted request as JSON.
func (_self *Page) Request() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_request(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the intercepted requests seen so far as JSON.
func (_self *Page) Requests() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_requests(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
func (_self *Page) Route(pattern string, action string, body *string, contentType *string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_route(
		_pointer,FfiConverterStringINSTANCE.Lower(pattern), FfiConverterStringINSTANCE.Lower(action), FfiConverterOptionalStringINSTANCE.Lower(body), FfiConverterOptionalStringINSTANCE.Lower(contentType)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Aborts every request matching `pattern`.
func (_self *Page) RouteAbort(pattern string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_route_abort(
		_pointer,FfiConverterStringINSTANCE.Lower(pattern)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Registers fulfill routes for every entry in a HAR file.
func (_self *Page) RouteFromHar(path string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_route_from_har(
		_pointer,FfiConverterStringINSTANCE.Lower(path)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Fulfills every request matching `pattern` with `body`.
func (_self *Page) RouteFulfill(pattern string, body string, contentType *string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_route_fulfill(
		_pointer,FfiConverterStringINSTANCE.Lower(pattern), FfiConverterStringINSTANCE.Lower(body), FfiConverterOptionalStringINSTANCE.Lower(contentType)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

func (_self *Page) Screenshot() ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []byte {
			return FfiConverterBytesINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_screenshot(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Page) ScreenshotFull() ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) []byte {
			return FfiConverterBytesINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_screenshot_full(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Selects options by value/label on the matching `<select>`.
func (_self *Page) SelectOption(selector string, valuesJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_select_option(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(valuesJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Enables or disables the HTTP cache.
func (_self *Page) SetCacheEnabled(enabled bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_cache_enabled(
		_pointer,FfiConverterBoolINSTANCE.Lower(enabled)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Replaces the document content.
func (_self *Page) SetContent(html string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_content(
		_pointer,FfiConverterStringINSTANCE.Lower(html)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Stores a default timeout (ms) for adapter compatibility.
func (_self *Page) SetDefaultTimeout(milliseconds float64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_default_timeout(
		_pointer,FfiConverterFloat64INSTANCE.Lower(milliseconds)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Enables or disables input drag interception.
func (_self *Page) SetDragInterception(enabled bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_drag_interception(
		_pointer,FfiConverterBoolINSTANCE.Lower(enabled)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Overrides media features (JSON array of `{name,value}`).
func (_self *Page) SetEmulatedMediaFeatures(featuresJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_emulated_media_features(
		_pointer,FfiConverterStringINSTANCE.Lower(featuresJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets extra HTTP headers for every request from this page.
func (_self *Page) SetExtraHttpHeaders(headersJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_extra_http_headers(
		_pointer,FfiConverterStringINSTANCE.Lower(headersJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets the files of the matching `<input type="file">`.
func (_self *Page) SetInputFiles(selector string, filesJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_input_files(
		_pointer,FfiConverterStringINSTANCE.Lower(selector), FfiConverterStringINSTANCE.Lower(filesJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Enables or disables JavaScript execution.
func (_self *Page) SetJavascriptEnabled(enabled bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_javascript_enabled(
		_pointer,FfiConverterBoolINSTANCE.Lower(enabled)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Toggles offline mode.
func (_self *Page) SetOffline(offline bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_offline(
		_pointer,FfiConverterBoolINSTANCE.Lower(offline)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Enables or disables request interception for this page.
func (_self *Page) SetRequestInterception(enabled bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_request_interception(
		_pointer,FfiConverterBoolINSTANCE.Lower(enabled)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Restores cookies + localStorage from a storage-state JSON object.
func (_self *Page) SetStorageState(stateJson string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_storage_state(
		_pointer,FfiConverterStringINSTANCE.Lower(stateJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Overrides `navigator.userAgent` for this page.
func (_self *Page) SetUserAgent(userAgent string, acceptLanguage *string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_user_agent(
		_pointer,FfiConverterStringINSTANCE.Lower(userAgent), FfiConverterOptionalStringINSTANCE.Lower(acceptLanguage)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Overrides the viewport size.
func (_self *Page) SetViewportSize(width uint64, height int64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_viewport_size(
		_pointer,FfiConverterUint64INSTANCE.Lower(width), FfiConverterInt64INSTANCE.Lower(height)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Moves and resizes the window.
func (_self *Page) SetWindowBounds(left int64, top int64, width int64, height int64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_window_bounds(
		_pointer,FfiConverterInt64INSTANCE.Lower(left), FfiConverterInt64INSTANCE.Lower(top), FfiConverterInt64INSTANCE.Lower(width), FfiConverterInt64INSTANCE.Lower(height)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Moves the window, preserving its size.
func (_self *Page) SetWindowPosition(x int64, y int64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_window_position(
		_pointer,FfiConverterInt64INSTANCE.Lower(x), FfiConverterInt64INSTANCE.Lower(y)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Resizes the window, preserving its position.
func (_self *Page) SetWindowSize(width int64, height int64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_window_size(
		_pointer,FfiConverterInt64INSTANCE.Lower(width), FfiConverterInt64INSTANCE.Lower(height)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
func (_self *Page) SetWindowState(state string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_set_window_state(
		_pointer,FfiConverterStringINSTANCE.Lower(state)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Starts a PNG screencast.
func (_self *Page) StartScreencast() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_start_screencast(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Starts CDP tracing on this page's session.
func (_self *Page) StartTracing() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_start_tracing(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Stops the screencast.
func (_self *Page) StopScreencast() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_stop_screencast(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Stops CDP tracing on this page's session.
func (_self *Page) StopTracing() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_stop_tracing(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns cookies + localStorage as a storage-state JSON object.
func (_self *Page) StorageState() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_storage_state(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// The CDP target id backing this page.
func (_self *Page) TargetId() string {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_page_target_id(
		_pointer,_uniffiStatus),
	}
	}))
}

// Returns the page title.
func (_self *Page) Title() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_title(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Dispatches a touch tap at (x, y).
func (_self *Page) TouchTap(x float64, y float64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_touch_tap(
		_pointer,FfiConverterFloat64INSTANCE.Lower(x), FfiConverterFloat64INSTANCE.Lower(y)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Removes the routes registered for `pattern`.
func (_self *Page) Unroute(pattern string) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_unroute(
		_pointer,FfiConverterStringINSTANCE.Lower(pattern)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Removes every route rule.
func (_self *Page) UnrouteAll() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_unroute_all(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Returns the current document URL.
func (_self *Page) Url() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_url(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Waits for the next CDP event named `event_name` and returns its params.
//
// The relevant domain is enabled first (best effort), so callers do not
// have to.
func (_self *Page) WaitForEvent(eventName string, timeoutMs uint64) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_wait_for_event(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName), FfiConverterUint64INSTANCE.Lower(timeoutMs)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// [`Page::wait_for_event`] with the default 30s timeout.
func (_self *Page) WaitForEventDefault(eventName string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_wait_for_event_default(
		_pointer,FfiConverterStringINSTANCE.Lower(eventName)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
func (_self *Page) WaitForFunction(expression string, timeoutMs uint64) error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_wait_for_function(
		_pointer,FfiConverterStringINSTANCE.Lower(expression), FfiConverterUint64INSTANCE.Lower(timeoutMs)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Waits for the page to finish loading.
func (_self *Page) WaitForNavigation() error {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 _, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) struct{} {
			C.ffi_xcelerate_rust_future_complete_void(handle, status)
			return struct{}{}
		},
		// liftFn
		func(_ struct{}) struct{} { return struct{}{} },
		C.uniffi_xcelerate_fn_method_page_wait_for_navigation(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_void(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_void(handle)
		},
	)

	if err == nil {
		return nil
	}

	return err 
}

// Waits for an element matching the selector to appear in the DOM.
func (_self *Page) WaitForSelector(selector string) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_wait_for_selector(
		_pointer,FfiConverterStringINSTANCE.Lower(selector)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Waits for the first XPath match to appear.
func (_self *Page) WaitForXpath(xpath string, timeoutMs uint64) (*Element, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
			res := C.ffi_xcelerate_rust_future_complete_u64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.uint64_t) *Element {
			return FfiConverterElementINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_wait_for_xpath(
		_pointer,FfiConverterStringINSTANCE.Lower(xpath), FfiConverterUint64INSTANCE.Lower(timeoutMs)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_u64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_u64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

func (_self *Page) WindowId() (int64, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) C.int64_t {
			res := C.ffi_xcelerate_rust_future_complete_i64(handle, status)
			return res
		},
		// liftFn
		func(ffi C.int64_t) int64 {
			return FfiConverterInt64INSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_window_id(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_i64(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_i64(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the window position as JSON (`{x,y}`).
func (_self *Page) WindowPosition() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_window_position(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
func (_self *Page) WindowRect() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_window_rect(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// Returns the window size as JSON (`{width,height}`).
func (_self *Page) WindowSize() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Page")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_page_window_size(
		_pointer,),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}
func (object *Page) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterPage struct {}

var FfiConverterPageINSTANCE = FfiConverterPage{}


func (c FfiConverterPage) Lift(handle C.uint64_t) *Page {
	result := &Page {
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_xcelerate_fn_clone_page(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_xcelerate_fn_free_page(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*Page).Destroy)
	return result
}

func (c FfiConverterPage) Read(reader io.Reader) *Page {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterPage) Lower(value *Page) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*Page")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterPage) Write(writer io.Writer, value *Page) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalPage(handle uint64) *Page {
	return FfiConverterPageINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalPage(value *Page) uint64 {
	return uint64(FfiConverterPageINSTANCE.Lower(value))
}

type FfiDestroyerPage struct {}

func (_ FfiDestroyerPage) Destroy(value *Page) {
		value.Destroy()
}



// A handle to an enabled plugin, exposed to every language.
type PluginHandleInterface interface {
	// Invoke an op with a JSON-encoded argument object; returns JSON.
	Invoke(op string, argsJson string) (string, error)
	// The ops this plugin exposes.
	Ops() []string
	// The plugin's name.
	PluginName() string
}
// A handle to an enabled plugin, exposed to every language.
type PluginHandle struct {
	ffiObject FfiObject
}




// Invoke an op with a JSON-encoded argument object; returns JSON.
func (_self *PluginHandle) Invoke(op string, argsJson string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*PluginHandle")
	defer _self.ffiObject.decrementPointer()
	 res, err :=uniffiRustCallAsync[*XcelerateError](
        FfiConverterXcelerateErrorINSTANCE,
		// completeFn
		func(handle C.uint64_t, status *C.RustCallStatus) RustBufferI {
			res := C.ffi_xcelerate_rust_future_complete_rust_buffer(handle, status)
			return GoRustBuffer {
		inner: res,
	}
		},
		// liftFn
		func(ffi RustBufferI) string {
			return FfiConverterStringINSTANCE.Lift(ffi)
		},
		C.uniffi_xcelerate_fn_method_pluginhandle_invoke(
		_pointer,FfiConverterStringINSTANCE.Lower(op), FfiConverterStringINSTANCE.Lower(argsJson)),
		// pollFn
		func (handle C.uint64_t, continuation C.UniffiRustFutureContinuationCallback, data C.uint64_t) {
			C.ffi_xcelerate_rust_future_poll_rust_buffer(handle, continuation, data)
		},
		// freeFn
		func (handle C.uint64_t) {
			C.ffi_xcelerate_rust_future_free_rust_buffer(handle)
		},
	)

	if err == nil {
		return res, nil
	}

	return res, err 
}

// The ops this plugin exposes.
func (_self *PluginHandle) Ops() []string {
	_pointer := _self.ffiObject.incrementPointer("*PluginHandle")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequenceStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_pluginhandle_ops(
		_pointer,_uniffiStatus),
	}
	}))
}

// The plugin's name.
func (_self *PluginHandle) PluginName() string {
	_pointer := _self.ffiObject.incrementPointer("*PluginHandle")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer {
		inner: C.uniffi_xcelerate_fn_method_pluginhandle_plugin_name(
		_pointer,_uniffiStatus),
	}
	}))
}
func (object *PluginHandle) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterPluginHandle struct {}

var FfiConverterPluginHandleINSTANCE = FfiConverterPluginHandle{}


func (c FfiConverterPluginHandle) Lift(handle C.uint64_t) *PluginHandle {
	result := &PluginHandle {
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_xcelerate_fn_clone_pluginhandle(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_xcelerate_fn_free_pluginhandle(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*PluginHandle).Destroy)
	return result
}

func (c FfiConverterPluginHandle) Read(reader io.Reader) *PluginHandle {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterPluginHandle) Lower(value *PluginHandle) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*PluginHandle")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterPluginHandle) Write(writer io.Writer, value *PluginHandle) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalPluginHandle(handle uint64) *PluginHandle {
	return FfiConverterPluginHandleINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalPluginHandle(value *PluginHandle) uint64 {
	return uint64(FfiConverterPluginHandleINSTANCE.Lower(value))
}

type FfiDestroyerPluginHandle struct {}

func (_ FfiDestroyerPluginHandle) Destroy(value *PluginHandle) {
		value.Destroy()
}



// Configuration for the Browser instance.
type BrowserConfig struct {
	// Whether to run the browser in headless mode.
	Headless bool
	// Whether to run the browser as a detached process.
	Detached bool
	// Optional path to the browser executable.
	ExecutablePath *string
	// First-party plugins to enable for this browser (for example
	// `["stealth", "human"]`). Default-deny: no plugin does anything unless
	// listed here (or enabled afterwards with `Browser::use_plugin`).
	Plugins *[]string
}

func (r *BrowserConfig) Destroy() {
		FfiDestroyerBool{}.Destroy(r.Headless);
		FfiDestroyerBool{}.Destroy(r.Detached);
		FfiDestroyerOptionalString{}.Destroy(r.ExecutablePath);
		FfiDestroyerOptionalSequenceString{}.Destroy(r.Plugins);
}

type FfiConverterBrowserConfig struct {}

var FfiConverterBrowserConfigINSTANCE = FfiConverterBrowserConfig{}

func (c FfiConverterBrowserConfig) Lift(rb RustBufferI) BrowserConfig {
	return LiftFromRustBuffer[BrowserConfig](c, rb)
}

func (c FfiConverterBrowserConfig) Read(reader io.Reader) BrowserConfig {
	return BrowserConfig {
			FfiConverterBoolINSTANCE.Read(reader),
			FfiConverterBoolINSTANCE.Read(reader),
			FfiConverterOptionalStringINSTANCE.Read(reader),
			FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBrowserConfig) Lower(value BrowserConfig) C.RustBuffer {
	return LowerIntoRustBuffer[BrowserConfig](c, value)
}

func (c FfiConverterBrowserConfig) LowerExternal(value BrowserConfig) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BrowserConfig](c, value))
}

func (c FfiConverterBrowserConfig) Write(writer io.Writer, value BrowserConfig) {
		FfiConverterBoolINSTANCE.Write(writer, value.Headless);
		FfiConverterBoolINSTANCE.Write(writer, value.Detached);
		FfiConverterOptionalStringINSTANCE.Write(writer, value.ExecutablePath);
		FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Plugins);
}

type FfiDestroyerBrowserConfig struct {}

func (_ FfiDestroyerBrowserConfig) Destroy(value BrowserConfig) {
	value.Destroy()
}
type XcelerateError struct {
	err error
}

// Convenience method to turn *XcelerateError into error
// Avoiding treating nil pointer as non nil error interface
func (err *XcelerateError) AsError() error {
	if err == nil {
		return nil
	} else {
		return err
	}
}

func (err XcelerateError) Error() string {
	return fmt.Sprintf("XcelerateError: %s", err.err.Error())
}

func (err XcelerateError) Unwrap() error {
	return err.err
}

// Err* are used for checking error type with `errors.Is`
var ErrXcelerateErrorWsError = fmt.Errorf("XcelerateErrorWsError")
var ErrXcelerateErrorSerdeError = fmt.Errorf("XcelerateErrorSerdeError")
var ErrXcelerateErrorCdpResponseError = fmt.Errorf("XcelerateErrorCdpResponseError")
var ErrXcelerateErrorHttpError = fmt.Errorf("XcelerateErrorHttpError")
var ErrXcelerateErrorNotFound = fmt.Errorf("XcelerateErrorNotFound")
var ErrXcelerateErrorInternalError = fmt.Errorf("XcelerateErrorInternalError")
var ErrXcelerateErrorUnsupported = fmt.Errorf("XcelerateErrorUnsupported")

// Variant structs
type XcelerateErrorWsError struct {
	message string
}
func NewXcelerateErrorWsError(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorWsError {} }
}

func (e XcelerateErrorWsError) destroy() {
}


func (err XcelerateErrorWsError) Error() string {
	return fmt.Sprintf("WsError: %s", err.message)
}

func (self XcelerateErrorWsError) Is(target error) bool {
	return target == ErrXcelerateErrorWsError
}
type XcelerateErrorSerdeError struct {
	message string
}
func NewXcelerateErrorSerdeError(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorSerdeError {} }
}

func (e XcelerateErrorSerdeError) destroy() {
}


func (err XcelerateErrorSerdeError) Error() string {
	return fmt.Sprintf("SerdeError: %s", err.message)
}

func (self XcelerateErrorSerdeError) Is(target error) bool {
	return target == ErrXcelerateErrorSerdeError
}
type XcelerateErrorCdpResponseError struct {
	message string
}
func NewXcelerateErrorCdpResponseError(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorCdpResponseError {} }
}

func (e XcelerateErrorCdpResponseError) destroy() {
}


func (err XcelerateErrorCdpResponseError) Error() string {
	return fmt.Sprintf("CdpResponseError: %s", err.message)
}

func (self XcelerateErrorCdpResponseError) Is(target error) bool {
	return target == ErrXcelerateErrorCdpResponseError
}
type XcelerateErrorHttpError struct {
	message string
}
func NewXcelerateErrorHttpError(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorHttpError {} }
}

func (e XcelerateErrorHttpError) destroy() {
}


func (err XcelerateErrorHttpError) Error() string {
	return fmt.Sprintf("HttpError: %s", err.message)
}

func (self XcelerateErrorHttpError) Is(target error) bool {
	return target == ErrXcelerateErrorHttpError
}
type XcelerateErrorNotFound struct {
	message string
}
func NewXcelerateErrorNotFound(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorNotFound {} }
}

func (e XcelerateErrorNotFound) destroy() {
}


func (err XcelerateErrorNotFound) Error() string {
	return fmt.Sprintf("NotFound: %s", err.message)
}

func (self XcelerateErrorNotFound) Is(target error) bool {
	return target == ErrXcelerateErrorNotFound
}
type XcelerateErrorInternalError struct {
	message string
}
func NewXcelerateErrorInternalError(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorInternalError {} }
}

func (e XcelerateErrorInternalError) destroy() {
}


func (err XcelerateErrorInternalError) Error() string {
	return fmt.Sprintf("InternalError: %s", err.message)
}

func (self XcelerateErrorInternalError) Is(target error) bool {
	return target == ErrXcelerateErrorInternalError
}
type XcelerateErrorUnsupported struct {
	message string
}
func NewXcelerateErrorUnsupported(
) *XcelerateError {
	return &XcelerateError { err: &XcelerateErrorUnsupported {} }
}

func (e XcelerateErrorUnsupported) destroy() {
}


func (err XcelerateErrorUnsupported) Error() string {
	return fmt.Sprintf("Unsupported: %s", err.message)
}

func (self XcelerateErrorUnsupported) Is(target error) bool {
	return target == ErrXcelerateErrorUnsupported
}

type FfiConverterXcelerateError struct{}

var FfiConverterXcelerateErrorINSTANCE = FfiConverterXcelerateError{}

func (c FfiConverterXcelerateError) Lift(eb RustBufferI) *XcelerateError {
	return LiftFromRustBuffer[*XcelerateError](c, eb)
}

func (c FfiConverterXcelerateError) Lower(value *XcelerateError) C.RustBuffer {
	return LowerIntoRustBuffer[*XcelerateError](c, value)
}

func (c FfiConverterXcelerateError) LowerExternal(value *XcelerateError) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*XcelerateError](c, value))
}

func (c FfiConverterXcelerateError) Read(reader io.Reader) *XcelerateError {
	errorID := readUint32(reader)

	message := FfiConverterStringINSTANCE.Read(reader)
	switch errorID {
	case 1:
		return &XcelerateError{ &XcelerateErrorWsError{message}}
	case 2:
		return &XcelerateError{ &XcelerateErrorSerdeError{message}}
	case 3:
		return &XcelerateError{ &XcelerateErrorCdpResponseError{message}}
	case 4:
		return &XcelerateError{ &XcelerateErrorHttpError{message}}
	case 5:
		return &XcelerateError{ &XcelerateErrorNotFound{message}}
	case 6:
		return &XcelerateError{ &XcelerateErrorInternalError{message}}
	case 7:
		return &XcelerateError{ &XcelerateErrorUnsupported{message}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterXcelerateError.Read()", errorID))
	}

	
}

func (c FfiConverterXcelerateError) Write(writer io.Writer, value *XcelerateError) {
	switch variantValue := value.err.(type) {
		case *XcelerateErrorWsError:
			writeInt32(writer, 1)
		case *XcelerateErrorSerdeError:
			writeInt32(writer, 2)
		case *XcelerateErrorCdpResponseError:
			writeInt32(writer, 3)
		case *XcelerateErrorHttpError:
			writeInt32(writer, 4)
		case *XcelerateErrorNotFound:
			writeInt32(writer, 5)
		case *XcelerateErrorInternalError:
			writeInt32(writer, 6)
		case *XcelerateErrorUnsupported:
			writeInt32(writer, 7)
		default:
			_ = variantValue
			panic(fmt.Sprintf("invalid error value `%v` in FfiConverterXcelerateError.Write", value))
	}
}

type FfiDestroyerXcelerateError struct {}

func (_ FfiDestroyerXcelerateError) Destroy(value *XcelerateError) {
	switch variantValue := value.err.(type) {
		case XcelerateErrorWsError:
			variantValue.destroy()
		case XcelerateErrorSerdeError:
			variantValue.destroy()
		case XcelerateErrorCdpResponseError:
			variantValue.destroy()
		case XcelerateErrorHttpError:
			variantValue.destroy()
		case XcelerateErrorNotFound:
			variantValue.destroy()
		case XcelerateErrorInternalError:
			variantValue.destroy()
		case XcelerateErrorUnsupported:
			variantValue.destroy()
		default:
			_ = variantValue
			panic(fmt.Sprintf("invalid error value `%v` in FfiDestroyerXcelerateError.Destroy", value))
	}
}



type FfiConverterOptionalString struct{}

var FfiConverterOptionalStringINSTANCE = FfiConverterOptionalString{}

func (c FfiConverterOptionalString) Lift(rb RustBufferI) *string {
	return LiftFromRustBuffer[*string](c, rb)
}

func (_ FfiConverterOptionalString) Read(reader io.Reader) *string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalString) Lower(value *string) C.RustBuffer {
	return LowerIntoRustBuffer[*string](c, value)
}

func (c FfiConverterOptionalString) LowerExternal(value *string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*string](c, value))
}

func (_ FfiConverterOptionalString) Write(writer io.Writer, value *string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalString struct {}

func (_ FfiDestroyerOptionalString) Destroy(value *string) {
	if value != nil {
		FfiDestroyerString{}.Destroy(*value)
	}
}


type FfiConverterOptionalSequenceString struct{}

var FfiConverterOptionalSequenceStringINSTANCE = FfiConverterOptionalSequenceString{}

func (c FfiConverterOptionalSequenceString) Lift(rb RustBufferI) *[]string {
	return LiftFromRustBuffer[*[]string](c, rb)
}

func (_ FfiConverterOptionalSequenceString) Read(reader io.Reader) *[]string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceString) Lower(value *[]string) C.RustBuffer {
	return LowerIntoRustBuffer[*[]string](c, value)
}

func (c FfiConverterOptionalSequenceString) LowerExternal(value *[]string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]string](c, value))
}

func (_ FfiConverterOptionalSequenceString) Write(writer io.Writer, value *[]string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceString struct {}

func (_ FfiDestroyerOptionalSequenceString) Destroy(value *[]string) {
	if value != nil {
		FfiDestroyerSequenceString{}.Destroy(*value)
	}
}


type FfiConverterSequenceString struct{}

var FfiConverterSequenceStringINSTANCE = FfiConverterSequenceString{}

func (c FfiConverterSequenceString) Lift(rb RustBufferI) []string {
	return LiftFromRustBuffer[[]string](c, rb)
}

func (c FfiConverterSequenceString) Read(reader io.Reader) []string {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]string, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterStringINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceString) Lower(value []string) C.RustBuffer {
	return LowerIntoRustBuffer[[]string](c, value)
}

func (c FfiConverterSequenceString) LowerExternal(value []string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]string](c, value))
}

func (c FfiConverterSequenceString) Write(writer io.Writer, value []string) {
	if len(value) > math.MaxInt32 {
		panic("[]string is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterStringINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceString struct {}

func (FfiDestroyerSequenceString) Destroy(sequence []string) {
	for _, value := range sequence {
		FfiDestroyerString{}.Destroy(value)	
	}
}


type FfiConverterSequenceElement struct{}

var FfiConverterSequenceElementINSTANCE = FfiConverterSequenceElement{}

func (c FfiConverterSequenceElement) Lift(rb RustBufferI) []*Element {
	return LiftFromRustBuffer[[]*Element](c, rb)
}

func (c FfiConverterSequenceElement) Read(reader io.Reader) []*Element {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*Element, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterElementINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceElement) Lower(value []*Element) C.RustBuffer {
	return LowerIntoRustBuffer[[]*Element](c, value)
}

func (c FfiConverterSequenceElement) LowerExternal(value []*Element) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]*Element](c, value))
}

func (c FfiConverterSequenceElement) Write(writer io.Writer, value []*Element) {
	if len(value) > math.MaxInt32 {
		panic("[]*Element is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterElementINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceElement struct {}

func (FfiDestroyerSequenceElement) Destroy(sequence []*Element) {
	for _, value := range sequence {
		FfiDestroyerElement{}.Destroy(value)	
	}
}


const (
	uniffiRustFuturePollReady      int8 = 0
	uniffiRustFuturePollMaybeReady int8 = 1
)

type rustFuturePollFunc func(C.uint64_t, C.UniffiRustFutureContinuationCallback, C.uint64_t)
type rustFutureCompleteFunc[T any] func(C.uint64_t, *C.RustCallStatus) T
type rustFutureFreeFunc func(C.uint64_t)

//export xcelerate_uniffiFutureContinuationCallback
func xcelerate_uniffiFutureContinuationCallback(data C.uint64_t, pollResult C.int8_t) {
	h := cgo.Handle(uintptr(data))
	waiter := h.Value().(chan int8)
	waiter <- int8(pollResult)
}

func uniffiRustCallAsync[E any, T any, F any](
	errConverter BufReader[E],
	completeFunc rustFutureCompleteFunc[F],
	liftFunc func(F) T,
	rustFuture C.uint64_t,
	pollFunc rustFuturePollFunc,
	freeFunc rustFutureFreeFunc,
) (T, E) {
	defer freeFunc(rustFuture)

	pollResult := int8(-1)
	waiter := make(chan int8, 1)

	chanHandle := cgo.NewHandle(waiter)
	defer chanHandle.Delete()

	for pollResult != uniffiRustFuturePollReady {
		pollFunc(
			rustFuture,
			(C.UniffiRustFutureContinuationCallback)(C.xcelerate_uniffiFutureContinuationCallback),
			C.uint64_t(chanHandle),
		)
		pollResult = <-waiter
	}

	var goValue T
	ffiValue, err := rustCallWithError(errConverter, func(status *C.RustCallStatus) F {
		return completeFunc(rustFuture, status)
	})
	if value := reflect.ValueOf(err); value.IsValid() && !value.IsZero() {
		return goValue, err
	}
	return liftFunc(ffiValue), err
}

//export xcelerate_uniffiFreeGorutine
func xcelerate_uniffiFreeGorutine(data C.uint64_t) {
	handle := cgo.Handle(uintptr(data))
	defer handle.Delete()

	guard := handle.Value().(chan struct{})
	guard <- struct{}{}
}

