/**
 * The structs here are equivalent to those in the WebAssembly spec here:
 * https://webassembly.github.io/spec/core/index.html
 */

use std::vec::Vec;

pub struct WasmModule {
	pub type_section: TypeSection,
	// TODO: more sections
}

pub trait Insert<T> {
	fn insert(value: T);
}

pub struct SearchableVec<T> {
	vec: Vec<T>,
	index: std::collections::BTreeMap<T, usize>,
}

pub struct TypeSection {
	data: SearchableVec<FuncType>,
}

pub struct FuncType {
	param_types: Vec<ValType>,
	result_types: Vec<ValType>,
}

pub enum ValType {
	I32,
	I64,
	F32,
	F64,
}

pub struct ImportSection {
	data: SearchableVec<Import>,
}

pub struct Import {
	module_name: String,
	entity_name: String,
	desc: ImportDesc,
}

pub enum ImportDesc {
	Func(u32),
	Table(TableType),
	Mem(MemType),
	Global(GlobalType),
}

pub struct TableType {
	elem_type: ElemType,
	limits: Limits,
}

pub enum ElemType {
	FuncRef,
}

pub enum Limits {
	Unbounded{min: u32},
	Bounded{min: u32, max: u32},
}

pub struct MemType {
	limits: Limits,
}

pub struct GlobalType {
	val_type: ValType,
	mutability: Mut,
}

pub enum Mut {
	Const,
	Var,
}



/**
 * Traits and implementations for serializing the WebAssembly module into bytecode.
 */

pub trait WasmSerialize {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8>;
}

pub trait LebSerialize { // LEB serialization
	fn leb_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8>;
}

pub trait LebSerialize5Byte { // special LEB serialization for i32 and u32 that will use exactly 5 bytes (for relocation purposes)
	fn leb_serialize_5_byte<'a, Rec: std::iter::Extend<u8> + std::iter::Extend<&'a u8>>(&self, receiver: &mut Rec);
}

impl WasmSerialize for WasmModule {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		receiver.extend(&[0x00, 0x61, 0x73, 0x6D]); // magic value "\0asm"
		receiver.extend(&[0x01, 0x00, 0x00, 0x00]); // WebAssembly version 1
		self.type_section.wasm_serialize(receiver);
	}
}

impl WasmSerialize for TypeSection {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		receiver.extend(&[1u8]); // the magic value for Type Section
		serialize_section(&self.data.vec, receiver);
	}
}

impl WasmSerialize for FuncType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		receiver.extend(&[0x60]); // magic value for FuncType
		self.param_types.wasm_serialize(receiver);
		self.result_types.wasm_serialize(receiver);
	}
}

impl WasmSerialize for ValType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		match *self {
			ValType::I32 => receiver.extend(&[0x7F]),
			ValType::I64 => receiver.extend(&[0x7E]),
			ValType::F32 => receiver.extend(&[0x7D]),
			ValType::F64 => receiver.extend(&[0x7C]),
		}
	}
}

fn serialize_section<T: WasmSerialize, Rec>(content: &T, receiver: &mut Rec)
	where
		for<'a> Rec: std::iter::Extend<&'a u8> {
	let mut buf = Vec::<u8>::new();
	content.wasm_serialize(&mut buf);
	(buf.len() as u32).leb_serialize(receiver);
	receiver.extend(&buf);
}





impl WasmSerialize for ImportSection {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		receiver.extend(&[2u8]); // the magic value for Type Section
		serialize_section(&self.data.vec, receiver);
	}
}

impl WasmSerialize for Import {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		self.module_name.wasm_serialize(receiver);
		self.entity_name.wasm_serialize(receiver);
		self.desc.wasm_serialize(receiver);
	}
}

impl WasmSerialize for ImportDesc {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		match *self {
			ImportDesc::Func(ref type_idx) => {
				receiver.extend(&[0x00]);
				type_idx.leb_serialize(receiver);
			}
			ImportDesc::Table(ref table_type) => {
				receiver.extend(&[0x01]);
				table_type.wasm_serialize(receiver);
			}
			ImportDesc::Mem(ref mem_type) => {
				receiver.extend(&[0x02]);
				mem_type.wasm_serialize(receiver);
			}
			ImportDesc::Global(ref global_type) => {
				receiver.extend(&[0x03]);
				global_type.wasm_serialize(receiver);
			}
		}
	}
}

impl WasmSerialize for TableType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		self.elem_type.wasm_serialize(receiver);
		self.limits.wasm_serialize(receiver);
	}
}


impl WasmSerialize for ElemType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		match *self {
			ElemType::FuncRef => receiver.extend(&[0x70]),
		}
	}
}

impl WasmSerialize for Limits {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		match *self {
			Limits::Unbounded{ref min} => {
				receiver.extend(&[0x00]);
				min.leb_serialize(receiver);
			}
			Limits::Bounded{ref min, ref max} => {
				receiver.extend(&[0x01]);
				min.leb_serialize(receiver);
				max.leb_serialize(receiver);
			}
		}
	}
}

impl WasmSerialize for MemType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		self.limits.wasm_serialize(receiver);
	}
}

impl WasmSerialize for GlobalType {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		self.val_type.wasm_serialize(receiver);
		self.mutability.wasm_serialize(receiver);
	}
}

impl WasmSerialize for Mut {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		match *self {
			Mut::Const => receiver.extend(&[0x00]),
			Mut::Var => receiver.extend(&[0x01]),
		}
	}
}






impl<T: WasmSerialize> WasmSerialize for Vec::<T> {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		(self.len() as u32).leb_serialize(receiver);
		for elem in self {
			elem.wasm_serialize(receiver);
		}
	}
}

impl WasmSerialize for str {
	fn wasm_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		let u8bytes = self.as_bytes();
		(u8bytes.len() as u32).leb_serialize(receiver);
		receiver.extend(u8bytes);
	}
}






impl LebSerialize for u32 {
	fn leb_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		// LEB128 conversion
		let mut x: u32 = *self;
		loop {
			let b: u8 = (x & 127u32) as u8;
			x >>= 7;
			if x != 0 { // still have more bytes
				receiver.extend(&[b | 128u8]); // set the 'more bytes' flag
			}
			else { // no more bytes
				receiver.extend(&[b]);
				break;
			}
		}
	}
}

impl LebSerialize for i32 {
	fn leb_serialize<Rec>(&self, receiver: &mut Rec)
		where
			for<'a> Rec: std::iter::Extend<&'a u8> {
		// LEB128 conversion
		let mut x: i32 = *self;
		loop {
			let b: u8 = (x & 127i32) as u8;
			x >>= 7; // this does sign extension (i.e. arithmetic shift right) when the left argument is signed
			if (x != 0 || b & 64u8 != 0) && (x != -1 || b & 64u8 == 0) { // still have more bytes
				// note about condition above: `b & 64u8` will become the sign bit if there are no more bytes, so we need to check if the sign bit is actually what we want
				receiver.extend(&[b | 128u8]); // set the 'more bytes' flag
			}
			else { // no more bytes
				receiver.extend(&[b]);
				break;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::wasmgen::*;
	fn wasm_serializer_wrapper<T: WasmSerialize + ?Sized>(val: &T) -> Vec::<u8> {
		let mut tmp = Vec::<u8>::new();
        val.wasm_serialize(&mut tmp);
		tmp
    }
	fn leb_serializer_wrapper<T: LebSerialize>(val: T) -> Vec::<u8> {
		let mut tmp = Vec::<u8>::new();
        val.leb_serialize(&mut tmp);
		tmp
    }

    #[test]
    fn wasm_serialize_vec() {
        assert_eq!(wasm_serializer_wrapper(&Vec::<ValType>::new()), [0]);
        assert_eq!(wasm_serializer_wrapper(&vec![ValType::I32]), [1, wasm_serializer_wrapper(&ValType::I32)[0]]);
        assert_eq!(wasm_serializer_wrapper(&vec![ValType::I32, ValType::F32]), [2, wasm_serializer_wrapper(&ValType::I32)[0], wasm_serializer_wrapper(&ValType::F32)[0]]);
    }

    #[test]
    fn wasm_serialize_string() {
        assert_eq!(wasm_serializer_wrapper(""), [0]);
        assert_eq!(wasm_serializer_wrapper("a"), [1, 'a' as u8]);
        assert_eq!(wasm_serializer_wrapper("test"), [4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]);
    }

    #[test]
    fn leb_serialize_unsigned() {
        assert_eq!(leb_serializer_wrapper(0u32), [0]);
        assert_eq!(leb_serializer_wrapper(1u32), [1]);
        assert_eq!(leb_serializer_wrapper(4u32), [4]);
        assert_eq!(leb_serializer_wrapper(127u32), [127]);
        assert_eq!(leb_serializer_wrapper(128u32), [128, 1]);
        assert_eq!(leb_serializer_wrapper(255u32), [255, 1]);
        assert_eq!(leb_serializer_wrapper(256u32), [128, 2]);
    }

    #[test]
    fn leb_serialize_signed() {
        assert_eq!(leb_serializer_wrapper(0i32), [0]);
        assert_eq!(leb_serializer_wrapper(1i32), [1]);
        assert_eq!(leb_serializer_wrapper(4i32), [4]);
        assert_eq!(leb_serializer_wrapper(63i32), [63]);
        assert_eq!(leb_serializer_wrapper(64i32), [192, 0]);
        assert_eq!(leb_serializer_wrapper(127i32), [255, 0]);
        assert_eq!(leb_serializer_wrapper(128i32), [128, 1]);
        assert_eq!(leb_serializer_wrapper(255i32), [255, 1]);
        assert_eq!(leb_serializer_wrapper(256i32), [128, 2]);

        assert_eq!(leb_serializer_wrapper(-1i32), [127]);
        assert_eq!(leb_serializer_wrapper(-4i32), [124]);
        assert_eq!(leb_serializer_wrapper(-63i32), [65]);
        assert_eq!(leb_serializer_wrapper(-64i32), [64]);
        assert_eq!(leb_serializer_wrapper(-65i32), [191, 127]);
        assert_eq!(leb_serializer_wrapper(-127i32), [129, 127]);
        assert_eq!(leb_serializer_wrapper(-128i32), [128, 127]);
        assert_eq!(leb_serializer_wrapper(-129i32), [255, 126]);
        assert_eq!(leb_serializer_wrapper(-255i32), [129, 126]);
        assert_eq!(leb_serializer_wrapper(-256i32), [128, 126]);
        assert_eq!(leb_serializer_wrapper(-257i32), [255, 125]);
    }
}

