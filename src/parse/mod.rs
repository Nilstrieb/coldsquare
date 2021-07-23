mod model;
pub use model::*;
use std::path::Prefix::Verbatim;

#[derive(Debug)]
pub struct ParseErr;

pub type Result<T> = std::result::Result<T, ParseErr>;

struct Data {
    data: Vec<u1>,
    pointer: usize,
}

pub fn parse_class_file(data: Vec<u1>) -> Result<ClassFile> {
    let mut data = Data { data, pointer: 0 };
    ClassFile::parse(&mut data)
}

impl From<Vec<u8>> for Data {
    fn from(data: Vec<u1>) -> Self {
        Self { data, pointer: 0 }
    }
}

impl Data {
    fn u1(&mut self) -> Result<u1> {
        let item = self.data.get(self.pointer).cloned();
        self.pointer += 1;
        item.ok_or(ParseErr)
    }

    fn u2(&mut self) -> Result<u2> {
        Ok(((self.u1()? as u2) << 8) | self.u1()? as u2)
    }

    fn u4(&mut self) -> Result<u4> {
        Ok(((self.u2()? as u4) << 16) | self.u2()? as u4)
    }

    fn last_u1(&self) -> Result<u1> {
        self.data.get(self.pointer - 1).cloned().ok_or(ParseErr)
    }

    fn last_u2(&self) -> Result<u2> {
        let last2u1 = self.data.get(self.pointer - 2).cloned().ok_or(ParseErr)?;
        Ok(((last2u1 as u2) << 8) | self.last_u1()? as u2)
    }

    fn last_u4(&self) -> Result<u4> {
        let last2u1 = self.data.get(self.pointer - 3).cloned().ok_or(ParseErr)?;
        let last3u1 = self.data.get(self.pointer - 4).cloned().ok_or(ParseErr)?;
        Ok(((last3u1 as u4) << 24) | ((last2u1 as u4) << 16) | self.last_u2()? as u4)
    }
}

pub trait Parse {
    fn parse(data: &mut Data) -> Result<Self>
    where
        Self: Sized;
}

pub trait ParseVec<T> {
    fn parse_vec(data: &mut Data, len: usize) -> Result<Self>
    where
        Self: Sized;
}

impl<T: Parse> ParseVec<T> for Vec<T> {
    fn parse_vec(data: &mut Data, len: usize) -> Result<Self> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::parse(data)?);
        }
        Ok(vec)
    }
}

macro_rules! parse_primitive {
    ($value:ident) => {
        impl Parse for $value {
            fn parse(data: &mut Data) -> Result<Self>
            where
                Self: Sized,
            {
                data.$value()
            }
        }
    };
}

parse_primitive!(u1);
parse_primitive!(u2);
parse_primitive!(u4);

impl Parse for ClassFile {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            magic: data.u4()?,
            minor_version: data.u2()?,
            major_version: data.u2()?,
            constant_pool_count: data.u2()?,
            constant_pool: Vec::parse_vec(data, data.last_u2()? as usize)?,
            access_flags: data.u2()?,
            this_class: data.u2()?,
            super_class: data.u2()?,
            interfaces_count: data.u2()?,
            interfaces: Vec::parse_vec(data, data.last_u2()? as usize)?,
            fields_count: data.u2()?,
            fields: Vec::parse_vec(data, data.last_u2()? as usize)?,
            method_count: data.u2()?,
            methods: Vec::parse_vec(data, data.last_u2()? as usize)?,
            attributes_count: data.u2()?,
            attributes: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

impl Parse for CpInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        let tag = data.u1()?;

        Ok(match tag {
            7 => Self::Class {
                tag,
                name_index: data.u2()?,
            },
            9 => Self::Fieldref {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            10 => Self::Methodref {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            11 => Self::InterfaceMethodref {
                tag,
                class_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            8 => Self::String {
                tag,
                string_index: data.u2()?,
            },
            3 => Self::Integer {
                tag,
                bytes: data.u4()?,
            },
            4 => Self::Float {
                tag,
                bytes: data.u4()?,
            },
            5 => Self::Long {
                tag,
                high_bytes: data.u4()?,
                low_bytes: data.u4()?,
            },
            6 => Self::Double {
                tag,
                high_bytes: data.u4()?,
                low_bytes: data.u4()?,
            },
            12 => Self::NameAndType {
                tag,
                name_index: data.u2()?,
                descriptor_index: data.u2()?,
            },
            1 => Self::Utf8 {
                tag,
                length: data.u2()?,
                bytes: Vec::parse_vec(data, data.last_u2()? as usize)?,
            },
            15 => Self::MethodHandle {
                tag,
                reference_kind: data.u1()?,
                reference_index: data.u2()?,
            },
            16 => Self::MethodType {
                tag,
                descriptor_index: data.u2()?,
            },
            18 => Self::InvokeDynamic {
                tag,
                bootstrap_method_attr_index: data.u2()?,
                name_and_type_index: data.u2()?,
            },
            _ => Err(ParseErr)?,
        })
    }
}

impl Parse for FieldInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.u2()?,
            descriptor_index: data.u2()?,
            attributes_count: data.u2()?,
            attributes: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

impl Parse for MethodInfo {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            access_flags: data.u2()?,
            name_index: data.u2()?,
            descriptor_index: data.u2()?,
            attributes_count: data.u2()?,
            attributes: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

impl Parse for Attribute {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            attribute_name_index: data.u2()?,
            attribute_length: data.u4()?,
            attribute_content: Vec::parse_vec(data, data.last_u4()? as usize)?,
        })
    }
}

impl Parse for AttributeCodeException {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.last_u2()?,
            end_pc: data.last_u2()?,
            handler_pc: data.last_u2()?,
            catch_type: data.last_u2()?,
        })
    }
}

impl Parse for StackMapFrame {
    fn parse(data: &mut Data) -> Result<Self> {
        let frame_type = data.u1()?;

        Ok(match frame_type {
            0..=63 => Self::SameFrame { frame_type },
            64..=127 => Self::SameLocals1StackItemFrame {
                frame_type,
                stack: VerificationTypeInfo::parse(data)?,
            },
            247 => Self::SameLocals1StackItemFrameExtended {
                frame_type,
                offset_delta: data.u2()?,
                stack: VerificationTypeInfo::parse(data)?,
            },
            246..=250 => Self::ChopFrame {
                frame_type,
                offset_delta: data.u2()?,
            },
            251 => Self::SameFrameExtended {
                frame_type,
                offset_delta: data.u2()?,
            },
            252..=254 => Self::AppendFrame {
                frame_type,
                offset_delta: data.u2()?,
                locals: Vec::parse_vec(data, data.last_u2()? as usize)?,
            },
            255 => Self::FullFrame {
                frame_type,
                offset_delta: data.u2()?,
                number_of_locals: data.u2()?,
                locals: Vec::parse_vec(data, data.last_u2()? as usize)?,
                number_of_stack_items: data.u2()?,
                stack: Vec::parse_vec(data, data.last_u2()? as usize)?,
            },
            _ => Err(ParseErr)?,
        })
    }
}

impl Parse for VerificationTypeInfo {
    fn parse(data: &mut Data) -> Result<Self>
    where
        Self: Sized,
    {
        let tag = data.u1()?;
        Ok(match tag {
            0 => Self::Top { tag },
            1 => Self::Integer { tag },
            2 => Self::Float { tag },
            4 => Self::Long { tag },
            3 => Self::Double { tag },
            5 => Self::Null { tag },
            6 => Self::UninitializedThis { tag },
            7 => Self::Object {
                tag,
                cpool_index: data.u2()?,
            },
            8 => Self::Uninitialized {
                tag,
                offset: data.u2()?,
            },
            _ => Err(ParseErr)?,
        })
    }
}

impl Parse for AttributeInnerClass {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            inner_class_info_index: data.u2()?,
            outer_class_info_index: data.u2()?,
            inner_class_name_index: data.u2()?,
            inner_class_access_flags: data.u2()?,
        })
    }
}

impl Parse for AttributeLineNumber {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.u2()?,
            line_number: data.u2()?,
        })
    }
}

impl Parse for AttributeLocalVariableTable {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            start_pc: data.u2()?,
            length: data.u2()?,
            name_index: data.u2()?,
            descriptor_or_signature_index: data.u2()?,
            index: data.u2()?,
        })
    }
}

impl Parse for Annotation {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            type_index: data.u2()?,
            num_element_value_pairs: data.u2()?,
            element_value_pairs: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

impl Parse for AnnotationElementValuePair {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            element_name_index: data.u2()?,
            element_name_name: AnnotationElementValue::parse(data)?,
        })
    }
}

impl Parse for AnnotationElementValue {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            tag: data.u1()?,
            value: AnnotationElementValueValue::parse(data)?,
        })
    }
}

impl Parse for AnnotationElementValueValue {
    fn parse(data: &mut Data) -> Result<Self> {
        let tag = data.last_u1()? as char;
        Ok(match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                Self::ConstValueIndex { index: data.u2()? }
            }
            'e' => Self::EnumConstValue {
                type_name_index: data.u2()?,
                const_name_index: data.u2()?,
            },
            'c' => Self::ClassInfoIndex { index: data.u2()? },
            '@' => Self::AnnotationValue {
                annotation: Box::new(Annotation::parse(data)?),
            },
            '[' => Self::ArrayValue {
                num_values: data.u2()?,
                values: Vec::parse_vec(data, data.last_u2()? as usize)?,
            },
            _ => Err(ParseErr)?,
        })
    }
}

impl Parse for ParameterAnnotation {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            num_annotations: data.u2()?,
            annotations: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

impl Parse for BootstrapMethod {
    fn parse(data: &mut Data) -> Result<Self> {
        Ok(Self {
            bootstrap_method_ref: data.u2()?,
            num_bootstrap_arguments: data.u2()?,
            bootstrap_arguments: Vec::parse_vec(data, data.last_u2()? as usize)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Data;

    #[test]
    fn data_u1() {
        let mut data = Data {
            data: vec![0xff, 0x00],
            pointer: 0,
        };
        assert_eq!(data.u1().unwrap(), 0xff);
        assert_eq!(data.u1().unwrap(), 0x00);
        assert_eq!(data.last_u1().unwrap(), 0x00);
    }

    #[test]
    fn data_u2() {
        let mut data = Data {
            data: vec![0xff, 0x33, 0x11, 0x00],
            pointer: 0,
        };
        assert_eq!(data.u2().unwrap(), 0xff33);
        assert_eq!(data.u2().unwrap(), 0x1100);
        assert_eq!(data.last_u2().unwrap(), 0x1100);
    }

    #[test]
    fn data_u4() {
        let mut data = Data {
            data: vec![0xff, 0x33, 0x11, 0x00],
            pointer: 0,
        };
        assert_eq!(data.u4().unwrap(), 0xff331100);
        assert_eq!(data.last_u4().unwrap(), 0xff331100);
    }
}
