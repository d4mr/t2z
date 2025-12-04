package t2z_uniffi

/*
#cgo LDFLAGS: -lt2z_uniffi
#include <t2z_uniffi.h>
*/
import "C"

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"runtime"
	"sync/atomic"
	"unsafe"
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

func RustBufferFromExternal(b RustBufferI) GoRustBuffer {
	return GoRustBuffer{
		inner: C.RustBuffer{
			capacity: C.uint64_t(b.Capacity()),
			len:      C.uint64_t(b.Len()),
			data:     (*C.uchar)(b.Data()),
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
	rustCall(func(status *C.RustCallStatus) bool {
		C.ffi_t2z_uniffi_rustbuffer_free(cb.inner, status)
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
	foreign := C.ForeignBytes{
		len:  C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}

	return rustCall(func(status *C.RustCallStatus) C.RustBuffer {
		return C.ffi_t2z_uniffi_rustbuffer_from_bytes(foreign, status)
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

func rustCallWithError[E any, U any](converter BufReader[*E], callback func(*C.RustCallStatus) U) (U, *E) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	err := checkCallStatus(converter, status)
	return returnValue, err
}

func checkCallStatus[E any](converter BufReader[*E], status C.RustCallStatus) *E {
	switch status.code {
	case 0:
		return nil
	case 1:
		return LiftFromRustBuffer(converter, GoRustBuffer{inner: status.errorBuf})
	case 2:
		// when the rust code sees a panic, it tries to construct a rustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{inner: status.errorBuf})))
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
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{
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
	bindingsContractVersion := 26
	// Get the scaffolding contract version by calling the into the dylib
	scaffoldingContractVersion := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint32_t {
		return C.ffi_t2z_uniffi_uniffi_contract_version()
	})
	if bindingsContractVersion != int(scaffoldingContractVersion) {
		// If this happens try cleaning and rebuilding your project
		panic("t2z_uniffi: UniFFI contract version mismatch")
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_append_signature()
		})
		if checksum != 44561 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_append_signature: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_combine_pczts()
		})
		if checksum != 42548 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_combine_pczts: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_finalize_and_extract()
		})
		if checksum != 39288 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_finalize_and_extract: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_finalize_and_extract_hex()
		})
		if checksum != 44713 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_finalize_and_extract_hex: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_get_sighash()
		})
		if checksum != 24122 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_get_sighash: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_is_proving_key_ready()
		})
		if checksum != 6630 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_is_proving_key_ready: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_prebuild_proving_key()
		})
		if checksum != 64108 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_prebuild_proving_key: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_propose_transaction()
		})
		if checksum != 43305 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_propose_transaction: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_prove_transaction()
		})
		if checksum != 25591 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_prove_transaction: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_sign_transparent_input()
		})
		if checksum != 24300 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_sign_transparent_input: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_verify_before_signing()
		})
		if checksum != 64908 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_verify_before_signing: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_func_version()
		})
		if checksum != 46598 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_func_version: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_method_uniffipczt_to_bytes()
		})
		if checksum != 18005 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_method_uniffipczt_to_bytes: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_method_uniffipczt_to_hex()
		})
		if checksum != 31942 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_method_uniffipczt_to_hex: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_constructor_uniffipczt_from_bytes()
		})
		if checksum != 29604 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_constructor_uniffipczt_from_bytes: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_t2z_uniffi_checksum_constructor_uniffipczt_from_hex()
		})
		if checksum != 48223 {
			// If this happens try cleaning and rebuilding your project
			panic("t2z_uniffi: uniffi_t2z_uniffi_checksum_constructor_uniffipczt_from_hex: UniFFI API checksum mismatch")
		}
	}
}

type FfiConverterUint32 struct{}

var FfiConverterUint32INSTANCE = FfiConverterUint32{}

func (FfiConverterUint32) Lower(value uint32) C.uint32_t {
	return C.uint32_t(value)
}

func (FfiConverterUint32) Write(writer io.Writer, value uint32) {
	writeUint32(writer, value)
}

func (FfiConverterUint32) Lift(value C.uint32_t) uint32 {
	return uint32(value)
}

func (FfiConverterUint32) Read(reader io.Reader) uint32 {
	return readUint32(reader)
}

type FfiDestroyerUint32 struct{}

func (FfiDestroyerUint32) Destroy(_ uint32) {}

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

type FfiDestroyerUint64 struct{}

func (FfiDestroyerUint64) Destroy(_ uint64) {}

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

type FfiDestroyerBool struct{}

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

type FfiDestroyerString struct{}

func (FfiDestroyerString) Destroy(_ string) {}

type FfiConverterBytes struct{}

var FfiConverterBytesINSTANCE = FfiConverterBytes{}

func (c FfiConverterBytes) Lower(value []byte) C.RustBuffer {
	return LowerIntoRustBuffer[[]byte](c, value)
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

type FfiDestroyerBytes struct{}

func (FfiDestroyerBytes) Destroy(_ []byte) {}

// Below is an implementation of synchronization requirements outlined in the link.
// https://github.com/mozilla/uniffi-rs/blob/0dc031132d9493ca812c3af6e7dd60ad2ea95bf0/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt#L31

type FfiObject struct {
	pointer       unsafe.Pointer
	callCounter   atomic.Int64
	cloneFunction func(unsafe.Pointer, *C.RustCallStatus) unsafe.Pointer
	freeFunction  func(unsafe.Pointer, *C.RustCallStatus)
	destroyed     atomic.Bool
}

func newFfiObject(
	pointer unsafe.Pointer,
	cloneFunction func(unsafe.Pointer, *C.RustCallStatus) unsafe.Pointer,
	freeFunction func(unsafe.Pointer, *C.RustCallStatus),
) FfiObject {
	return FfiObject{
		pointer:       pointer,
		cloneFunction: cloneFunction,
		freeFunction:  freeFunction,
	}
}

func (ffiObject *FfiObject) incrementPointer(debugName string) unsafe.Pointer {
	for {
		counter := ffiObject.callCounter.Load()
		if counter <= -1 {
			panic(fmt.Errorf("%v object has already been destroyed", debugName))
		}
		if counter == math.MaxInt64 {
			panic(fmt.Errorf("%v object call counter would overflow", debugName))
		}
		if ffiObject.callCounter.CompareAndSwap(counter, counter+1) {
			break
		}
	}

	return rustCall(func(status *C.RustCallStatus) unsafe.Pointer {
		return ffiObject.cloneFunction(ffiObject.pointer, status)
	})
}

func (ffiObject *FfiObject) decrementPointer() {
	if ffiObject.callCounter.Add(-1) == -1 {
		ffiObject.freeRustArcPtr()
	}
}

func (ffiObject *FfiObject) destroy() {
	if ffiObject.destroyed.CompareAndSwap(false, true) {
		if ffiObject.callCounter.Add(-1) == -1 {
			ffiObject.freeRustArcPtr()
		}
	}
}

func (ffiObject *FfiObject) freeRustArcPtr() {
	rustCall(func(status *C.RustCallStatus) int32 {
		ffiObject.freeFunction(ffiObject.pointer, status)
		return 0
	})
}

type UniffiPcztInterface interface {
	// Serializes the PCZT to bytes
	ToBytes() []byte
	// Serializes the PCZT to hex string
	ToHex() string
}
type UniffiPczt struct {
	ffiObject FfiObject
}

// Creates a UniffiPczt from raw bytes
func UniffiPcztFromBytes(bytes []byte) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_constructor_uniffipczt_from_bytes(FfiConverterBytesINSTANCE.Lower(bytes), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Creates a UniffiPczt from hex string
func UniffiPcztFromHex(hexString string) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_constructor_uniffipczt_from_hex(FfiConverterStringINSTANCE.Lower(hexString), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Serializes the PCZT to bytes
func (_self *UniffiPczt) ToBytes() []byte {
	_pointer := _self.ffiObject.incrementPointer("*UniffiPczt")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterBytesINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_method_uniffipczt_to_bytes(
				_pointer, _uniffiStatus),
		}
	}))
}

// Serializes the PCZT to hex string
func (_self *UniffiPczt) ToHex() string {
	_pointer := _self.ffiObject.incrementPointer("*UniffiPczt")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_method_uniffipczt_to_hex(
				_pointer, _uniffiStatus),
		}
	}))
}
func (object *UniffiPczt) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterUniffiPczt struct{}

var FfiConverterUniffiPcztINSTANCE = FfiConverterUniffiPczt{}

func (c FfiConverterUniffiPczt) Lift(pointer unsafe.Pointer) *UniffiPczt {
	result := &UniffiPczt{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) unsafe.Pointer {
				return C.uniffi_t2z_uniffi_fn_clone_uniffipczt(pointer, status)
			},
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_t2z_uniffi_fn_free_uniffipczt(pointer, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*UniffiPczt).Destroy)
	return result
}

func (c FfiConverterUniffiPczt) Read(reader io.Reader) *UniffiPczt {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterUniffiPczt) Lower(value *UniffiPczt) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*UniffiPczt")
	defer value.ffiObject.decrementPointer()
	return pointer

}

func (c FfiConverterUniffiPczt) Write(writer io.Writer, value *UniffiPczt) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerUniffiPczt struct{}

func (_ FfiDestroyerUniffiPczt) Destroy(value *UniffiPczt) {
	value.Destroy()
}

// Expected transaction output for verification
// Per spec: verify_before_signing takes expected_change: [TxOut]
type UniffiExpectedTxOut struct {
	// Address (transparent or Orchard unified address)
	Address string
	// Value in zatoshis
	Amount uint64
}

func (r *UniffiExpectedTxOut) Destroy() {
	FfiDestroyerString{}.Destroy(r.Address)
	FfiDestroyerUint64{}.Destroy(r.Amount)
}

type FfiConverterUniffiExpectedTxOut struct{}

var FfiConverterUniffiExpectedTxOutINSTANCE = FfiConverterUniffiExpectedTxOut{}

func (c FfiConverterUniffiExpectedTxOut) Lift(rb RustBufferI) UniffiExpectedTxOut {
	return LiftFromRustBuffer[UniffiExpectedTxOut](c, rb)
}

func (c FfiConverterUniffiExpectedTxOut) Read(reader io.Reader) UniffiExpectedTxOut {
	return UniffiExpectedTxOut{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterUint64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUniffiExpectedTxOut) Lower(value UniffiExpectedTxOut) C.RustBuffer {
	return LowerIntoRustBuffer[UniffiExpectedTxOut](c, value)
}

func (c FfiConverterUniffiExpectedTxOut) Write(writer io.Writer, value UniffiExpectedTxOut) {
	FfiConverterStringINSTANCE.Write(writer, value.Address)
	FfiConverterUint64INSTANCE.Write(writer, value.Amount)
}

type FfiDestroyerUniffiExpectedTxOut struct{}

func (_ FfiDestroyerUniffiExpectedTxOut) Destroy(value UniffiExpectedTxOut) {
	value.Destroy()
}

type UniffiPayment struct {
	// Address (transparent P2PKH/P2SH or unified with Orchard)
	Address string
	// Value in zatoshis
	Amount uint64
	// Optional memo (hex encoded, max 512 bytes)
	Memo *string
	// Optional label
	Label *string
}

func (r *UniffiPayment) Destroy() {
	FfiDestroyerString{}.Destroy(r.Address)
	FfiDestroyerUint64{}.Destroy(r.Amount)
	FfiDestroyerOptionalString{}.Destroy(r.Memo)
	FfiDestroyerOptionalString{}.Destroy(r.Label)
}

type FfiConverterUniffiPayment struct{}

var FfiConverterUniffiPaymentINSTANCE = FfiConverterUniffiPayment{}

func (c FfiConverterUniffiPayment) Lift(rb RustBufferI) UniffiPayment {
	return LiftFromRustBuffer[UniffiPayment](c, rb)
}

func (c FfiConverterUniffiPayment) Read(reader io.Reader) UniffiPayment {
	return UniffiPayment{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterUint64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUniffiPayment) Lower(value UniffiPayment) C.RustBuffer {
	return LowerIntoRustBuffer[UniffiPayment](c, value)
}

func (c FfiConverterUniffiPayment) Write(writer io.Writer, value UniffiPayment) {
	FfiConverterStringINSTANCE.Write(writer, value.Address)
	FfiConverterUint64INSTANCE.Write(writer, value.Amount)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Memo)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerUniffiPayment struct{}

func (_ FfiDestroyerUniffiPayment) Destroy(value UniffiPayment) {
	value.Destroy()
}

// Transaction request per ZIP 321 specification
// See: https://zips.z.cash/zip-0321
type UniffiTransactionRequest struct {
	// List of payments (ZIP 321 format)
	Payments []UniffiPayment
}

func (r *UniffiTransactionRequest) Destroy() {
	FfiDestroyerSequenceUniffiPayment{}.Destroy(r.Payments)
}

type FfiConverterUniffiTransactionRequest struct{}

var FfiConverterUniffiTransactionRequestINSTANCE = FfiConverterUniffiTransactionRequest{}

func (c FfiConverterUniffiTransactionRequest) Lift(rb RustBufferI) UniffiTransactionRequest {
	return LiftFromRustBuffer[UniffiTransactionRequest](c, rb)
}

func (c FfiConverterUniffiTransactionRequest) Read(reader io.Reader) UniffiTransactionRequest {
	return UniffiTransactionRequest{
		FfiConverterSequenceUniffiPaymentINSTANCE.Read(reader),
	}
}

func (c FfiConverterUniffiTransactionRequest) Lower(value UniffiTransactionRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UniffiTransactionRequest](c, value)
}

func (c FfiConverterUniffiTransactionRequest) Write(writer io.Writer, value UniffiTransactionRequest) {
	FfiConverterSequenceUniffiPaymentINSTANCE.Write(writer, value.Payments)
}

type FfiDestroyerUniffiTransactionRequest struct{}

func (_ FfiDestroyerUniffiTransactionRequest) Destroy(value UniffiTransactionRequest) {
	value.Destroy()
}

type UniffiTransparentInput struct {
	// Public key (33 bytes as hex string)
	Pubkey string
	// Previous transaction ID (32 bytes as hex string)
	PrevoutTxid string
	// Previous output index
	PrevoutIndex uint32
	// Value in zatoshis
	Value uint64
	// Script pubkey (hex encoded)
	ScriptPubkey string
	// Optional sequence number
	Sequence *uint32
}

func (r *UniffiTransparentInput) Destroy() {
	FfiDestroyerString{}.Destroy(r.Pubkey)
	FfiDestroyerString{}.Destroy(r.PrevoutTxid)
	FfiDestroyerUint32{}.Destroy(r.PrevoutIndex)
	FfiDestroyerUint64{}.Destroy(r.Value)
	FfiDestroyerString{}.Destroy(r.ScriptPubkey)
	FfiDestroyerOptionalUint32{}.Destroy(r.Sequence)
}

type FfiConverterUniffiTransparentInput struct{}

var FfiConverterUniffiTransparentInputINSTANCE = FfiConverterUniffiTransparentInput{}

func (c FfiConverterUniffiTransparentInput) Lift(rb RustBufferI) UniffiTransparentInput {
	return LiftFromRustBuffer[UniffiTransparentInput](c, rb)
}

func (c FfiConverterUniffiTransparentInput) Read(reader io.Reader) UniffiTransparentInput {
	return UniffiTransparentInput{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterUint32INSTANCE.Read(reader),
		FfiConverterUint64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalUint32INSTANCE.Read(reader),
	}
}

func (c FfiConverterUniffiTransparentInput) Lower(value UniffiTransparentInput) C.RustBuffer {
	return LowerIntoRustBuffer[UniffiTransparentInput](c, value)
}

func (c FfiConverterUniffiTransparentInput) Write(writer io.Writer, value UniffiTransparentInput) {
	FfiConverterStringINSTANCE.Write(writer, value.Pubkey)
	FfiConverterStringINSTANCE.Write(writer, value.PrevoutTxid)
	FfiConverterUint32INSTANCE.Write(writer, value.PrevoutIndex)
	FfiConverterUint64INSTANCE.Write(writer, value.Value)
	FfiConverterStringINSTANCE.Write(writer, value.ScriptPubkey)
	FfiConverterOptionalUint32INSTANCE.Write(writer, value.Sequence)
}

type FfiDestroyerUniffiTransparentInput struct{}

func (_ FfiDestroyerUniffiTransparentInput) Destroy(value UniffiTransparentInput) {
	value.Destroy()
}

type UniffiError struct {
	err error
}

// Convience method to turn *UniffiError into error
// Avoiding treating nil pointer as non nil error interface
func (err *UniffiError) AsError() error {
	if err == nil {
		return nil
	} else {
		return err
	}
}

func (err UniffiError) Error() string {
	return fmt.Sprintf("UniffiError: %s", err.err.Error())
}

func (err UniffiError) Unwrap() error {
	return err.err
}

// Err* are used for checking error type with `errors.Is`
var ErrUniffiErrorError = fmt.Errorf("UniffiErrorError")

// Variant structs
type UniffiErrorError struct {
	Msg string
}

func NewUniffiErrorError(
	msg string,
) *UniffiError {
	return &UniffiError{err: &UniffiErrorError{
		Msg: msg}}
}

func (e UniffiErrorError) destroy() {
	FfiDestroyerString{}.Destroy(e.Msg)
}

func (err UniffiErrorError) Error() string {
	return fmt.Sprint("Error",
		": ",

		"Msg=",
		err.Msg,
	)
}

func (self UniffiErrorError) Is(target error) bool {
	return target == ErrUniffiErrorError
}

type FfiConverterUniffiError struct{}

var FfiConverterUniffiErrorINSTANCE = FfiConverterUniffiError{}

func (c FfiConverterUniffiError) Lift(eb RustBufferI) *UniffiError {
	return LiftFromRustBuffer[*UniffiError](c, eb)
}

func (c FfiConverterUniffiError) Lower(value *UniffiError) C.RustBuffer {
	return LowerIntoRustBuffer[*UniffiError](c, value)
}

func (c FfiConverterUniffiError) Read(reader io.Reader) *UniffiError {
	errorID := readUint32(reader)

	switch errorID {
	case 1:
		return &UniffiError{&UniffiErrorError{
			Msg: FfiConverterStringINSTANCE.Read(reader),
		}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterUniffiError.Read()", errorID))
	}
}

func (c FfiConverterUniffiError) Write(writer io.Writer, value *UniffiError) {
	switch variantValue := value.err.(type) {
	case *UniffiErrorError:
		writeInt32(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Msg)
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiConverterUniffiError.Write", value))
	}
}

type FfiDestroyerUniffiError struct{}

func (_ FfiDestroyerUniffiError) Destroy(value *UniffiError) {
	switch variantValue := value.err.(type) {
	case UniffiErrorError:
		variantValue.destroy()
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiDestroyerUniffiError.Destroy", value))
	}
}

type FfiConverterOptionalUint32 struct{}

var FfiConverterOptionalUint32INSTANCE = FfiConverterOptionalUint32{}

func (c FfiConverterOptionalUint32) Lift(rb RustBufferI) *uint32 {
	return LiftFromRustBuffer[*uint32](c, rb)
}

func (_ FfiConverterOptionalUint32) Read(reader io.Reader) *uint32 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUint32INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUint32) Lower(value *uint32) C.RustBuffer {
	return LowerIntoRustBuffer[*uint32](c, value)
}

func (_ FfiConverterOptionalUint32) Write(writer io.Writer, value *uint32) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUint32INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUint32 struct{}

func (_ FfiDestroyerOptionalUint32) Destroy(value *uint32) {
	if value != nil {
		FfiDestroyerUint32{}.Destroy(*value)
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

func (_ FfiConverterOptionalString) Write(writer io.Writer, value *string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalString struct{}

func (_ FfiDestroyerOptionalString) Destroy(value *string) {
	if value != nil {
		FfiDestroyerString{}.Destroy(*value)
	}
}

type FfiConverterSequenceUniffiPczt struct{}

var FfiConverterSequenceUniffiPcztINSTANCE = FfiConverterSequenceUniffiPczt{}

func (c FfiConverterSequenceUniffiPczt) Lift(rb RustBufferI) []*UniffiPczt {
	return LiftFromRustBuffer[[]*UniffiPczt](c, rb)
}

func (c FfiConverterSequenceUniffiPczt) Read(reader io.Reader) []*UniffiPczt {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*UniffiPczt, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterUniffiPcztINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceUniffiPczt) Lower(value []*UniffiPczt) C.RustBuffer {
	return LowerIntoRustBuffer[[]*UniffiPczt](c, value)
}

func (c FfiConverterSequenceUniffiPczt) Write(writer io.Writer, value []*UniffiPczt) {
	if len(value) > math.MaxInt32 {
		panic("[]*UniffiPczt is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterUniffiPcztINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceUniffiPczt struct{}

func (FfiDestroyerSequenceUniffiPczt) Destroy(sequence []*UniffiPczt) {
	for _, value := range sequence {
		FfiDestroyerUniffiPczt{}.Destroy(value)
	}
}

type FfiConverterSequenceUniffiExpectedTxOut struct{}

var FfiConverterSequenceUniffiExpectedTxOutINSTANCE = FfiConverterSequenceUniffiExpectedTxOut{}

func (c FfiConverterSequenceUniffiExpectedTxOut) Lift(rb RustBufferI) []UniffiExpectedTxOut {
	return LiftFromRustBuffer[[]UniffiExpectedTxOut](c, rb)
}

func (c FfiConverterSequenceUniffiExpectedTxOut) Read(reader io.Reader) []UniffiExpectedTxOut {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]UniffiExpectedTxOut, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterUniffiExpectedTxOutINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceUniffiExpectedTxOut) Lower(value []UniffiExpectedTxOut) C.RustBuffer {
	return LowerIntoRustBuffer[[]UniffiExpectedTxOut](c, value)
}

func (c FfiConverterSequenceUniffiExpectedTxOut) Write(writer io.Writer, value []UniffiExpectedTxOut) {
	if len(value) > math.MaxInt32 {
		panic("[]UniffiExpectedTxOut is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterUniffiExpectedTxOutINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceUniffiExpectedTxOut struct{}

func (FfiDestroyerSequenceUniffiExpectedTxOut) Destroy(sequence []UniffiExpectedTxOut) {
	for _, value := range sequence {
		FfiDestroyerUniffiExpectedTxOut{}.Destroy(value)
	}
}

type FfiConverterSequenceUniffiPayment struct{}

var FfiConverterSequenceUniffiPaymentINSTANCE = FfiConverterSequenceUniffiPayment{}

func (c FfiConverterSequenceUniffiPayment) Lift(rb RustBufferI) []UniffiPayment {
	return LiftFromRustBuffer[[]UniffiPayment](c, rb)
}

func (c FfiConverterSequenceUniffiPayment) Read(reader io.Reader) []UniffiPayment {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]UniffiPayment, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterUniffiPaymentINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceUniffiPayment) Lower(value []UniffiPayment) C.RustBuffer {
	return LowerIntoRustBuffer[[]UniffiPayment](c, value)
}

func (c FfiConverterSequenceUniffiPayment) Write(writer io.Writer, value []UniffiPayment) {
	if len(value) > math.MaxInt32 {
		panic("[]UniffiPayment is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterUniffiPaymentINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceUniffiPayment struct{}

func (FfiDestroyerSequenceUniffiPayment) Destroy(sequence []UniffiPayment) {
	for _, value := range sequence {
		FfiDestroyerUniffiPayment{}.Destroy(value)
	}
}

type FfiConverterSequenceUniffiTransparentInput struct{}

var FfiConverterSequenceUniffiTransparentInputINSTANCE = FfiConverterSequenceUniffiTransparentInput{}

func (c FfiConverterSequenceUniffiTransparentInput) Lift(rb RustBufferI) []UniffiTransparentInput {
	return LiftFromRustBuffer[[]UniffiTransparentInput](c, rb)
}

func (c FfiConverterSequenceUniffiTransparentInput) Read(reader io.Reader) []UniffiTransparentInput {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]UniffiTransparentInput, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterUniffiTransparentInputINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceUniffiTransparentInput) Lower(value []UniffiTransparentInput) C.RustBuffer {
	return LowerIntoRustBuffer[[]UniffiTransparentInput](c, value)
}

func (c FfiConverterSequenceUniffiTransparentInput) Write(writer io.Writer, value []UniffiTransparentInput) {
	if len(value) > math.MaxInt32 {
		panic("[]UniffiTransparentInput is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterUniffiTransparentInputINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceUniffiTransparentInput struct{}

func (FfiDestroyerSequenceUniffiTransparentInput) Destroy(sequence []UniffiTransparentInput) {
	for _, value := range sequence {
		FfiDestroyerUniffiTransparentInput{}.Destroy(value)
	}
}

// Appends a signature to a transparent input
//
// # Arguments
// * `pczt` - The PCZT
// * `input_index` - Index of the input to sign
// * `pubkey_hex` - Compressed public key (33 bytes, hex)
// * `signature_hex` - DER-encoded ECDSA signature (hex)
func AppendSignature(pczt *UniffiPczt, inputIndex uint32, pubkeyHex string, signatureHex string) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_func_append_signature(FfiConverterUniffiPcztINSTANCE.Lower(pczt), FfiConverterUint32INSTANCE.Lower(inputIndex), FfiConverterStringINSTANCE.Lower(pubkeyHex), FfiConverterStringINSTANCE.Lower(signatureHex), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Combines multiple PCZTs into one
func CombinePczts(pcztList []*UniffiPczt) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_func_combine_pczts(FfiConverterSequenceUniffiPcztINSTANCE.Lower(pcztList), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Finalizes the PCZT and extracts the transaction bytes
func FinalizeAndExtract(pczt *UniffiPczt) ([]byte, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_func_finalize_and_extract(FfiConverterUniffiPcztINSTANCE.Lower(pczt), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []byte
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBytesINSTANCE.Lift(_uniffiRV), nil
	}
}

// Finalizes the PCZT and extracts the transaction as hex string
func FinalizeAndExtractHex(pczt *UniffiPczt) (string, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_func_finalize_and_extract_hex(FfiConverterUniffiPcztINSTANCE.Lower(pczt), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue string
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStringINSTANCE.Lift(_uniffiRV), nil
	}
}

// Gets the sighash for a transparent input
//
// The returned sighash should be signed externally, then the signature
// appended using append_signature.
func GetSighash(pczt *UniffiPczt, inputIndex uint32) (string, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_func_get_sighash(FfiConverterUniffiPcztINSTANCE.Lower(pczt), FfiConverterUint32INSTANCE.Lower(inputIndex), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue string
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStringINSTANCE.Lift(_uniffiRV), nil
	}
}

// Check if the proving key has been built and cached
func IsProvingKeyReady() bool {
	return FfiConverterBoolINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.int8_t {
		return C.uniffi_t2z_uniffi_fn_func_is_proving_key_ready(_uniffiStatus)
	}))
}

// Pre-build the Orchard proving key
//
// Call this at application startup to avoid blocking during transaction proving.
func PrebuildProvingKey() {
	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_t2z_uniffi_fn_func_prebuild_proving_key(_uniffiStatus)
		return false
	})
}

// Proposes a transaction from transparent inputs to transparent and/or shielded outputs
//
// # Arguments
// * `inputs_to_spend` - UTXOs to spend
// * `transaction_request` - ZIP 321 payment request (payments only)
// * `change_address` - Optional address for change (transparent or Orchard)
// * `network` - "mainnet" or "testnet"
// * `expiry_height` - Transaction expiry height
func ProposeTransaction(inputsToSpend []UniffiTransparentInput, transactionRequest UniffiTransactionRequest, changeAddress *string, network string, expiryHeight uint32) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_func_propose_transaction(FfiConverterSequenceUniffiTransparentInputINSTANCE.Lower(inputsToSpend), FfiConverterUniffiTransactionRequestINSTANCE.Lower(transactionRequest), FfiConverterOptionalStringINSTANCE.Lower(changeAddress), FfiConverterStringINSTANCE.Lower(network), FfiConverterUint32INSTANCE.Lower(expiryHeight), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Proves a transaction (builds proving key automatically, ~10 seconds first call)
//
// This uses Halo 2, which requires NO external downloads or trusted setup.
// The proving key is built programmatically and cached for subsequent calls.
func ProveTransaction(pczt *UniffiPczt) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_func_prove_transaction(FfiConverterUniffiPcztINSTANCE.Lower(pczt), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Signs a transparent input with the provided private key
func SignTransparentInput(pczt *UniffiPczt, inputIndex uint32, secretKeyHex string) (*UniffiPczt, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_t2z_uniffi_fn_func_sign_transparent_input(FfiConverterUniffiPcztINSTANCE.Lower(pczt), FfiConverterUint32INSTANCE.Lower(inputIndex), FfiConverterStringINSTANCE.Lower(secretKeyHex), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *UniffiPczt
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUniffiPcztINSTANCE.Lift(_uniffiRV), nil
	}
}

// Verifies the PCZT matches the original transaction request before signing
//
// Per spec: this may be skipped if the same entity created and is signing the PCZT
// with no third-party involvement.
//
// # Arguments
// * `pczt` - The PCZT to verify
// * `transaction_request` - Original ZIP 321 payment request
// * `expected_change` - List of expected change outputs (address + amount)
func VerifyBeforeSigning(pczt *UniffiPczt, transactionRequest UniffiTransactionRequest, expectedChange []UniffiExpectedTxOut) error {
	_, _uniffiErr := rustCallWithError[UniffiError](FfiConverterUniffiError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_t2z_uniffi_fn_func_verify_before_signing(FfiConverterUniffiPcztINSTANCE.Lower(pczt), FfiConverterUniffiTransactionRequestINSTANCE.Lower(transactionRequest), FfiConverterSequenceUniffiExpectedTxOutINSTANCE.Lower(expectedChange), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Gets the version of the library
func Version() string {
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_t2z_uniffi_fn_func_version(_uniffiStatus),
		}
	}))
}
