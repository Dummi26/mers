use crate::lang::{
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
};

/// any message implements this trait. reponding to the message with A generates the response of type B.
pub trait RespondableMessage: MessageResponse {
    type With;
    type Response: MessageResponse;
    fn respond(self, with: Self::With) -> Self::Response;
}

/// any message or response implements this trait
pub trait MessageResponse: ByteData + ByteDataA {
    fn messagetype_id() -> u32;
    fn msgtype_id(&self) -> u32 {
        Self::messagetype_id()
    }
}

pub trait ByteData: Sized {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read;
}
/// for things like &str, which can't be created easily (String exists for that purpose), but can still be converted to bytes.
pub trait ByteDataA {
    fn as_byte_data(&self, vec: &mut Vec<u8>);
    fn as_byte_data_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.as_byte_data(&mut vec);
        vec
    }
}

#[derive(Debug)]
pub enum Message {
    RunFunction(run_function::Message),
}
impl ByteData for Message {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut type_id = u32::from_byte_data(data)?;
        Ok(match type_id {
            0 => Self::RunFunction(ByteData::from_byte_data(data)?),
            other => unreachable!("read unknown type_id byte for message!"),
        })
    }
}
impl From<run_function::Message> for Message {
    fn from(value: run_function::Message) -> Self {
        Self::RunFunction(value)
    }
}

// implementations for the message/response pairs

pub mod run_function {
    use crate::lang::val_data::VData;

    use super::{ByteData, ByteDataA, MessageResponse, RespondableMessage};

    #[derive(Debug)]
    pub struct Message {
        pub function_id: u64,
        pub args: Vec<VData>,
    }
    pub struct Response {
        pub result: VData,
    }
    impl RespondableMessage for Message {
        type With = VData;
        type Response = Response;
        fn respond(self, with: Self::With) -> Self::Response {
            Response { result: with }
        }
    }
    impl MessageResponse for Message {
        fn messagetype_id() -> u32 {
            0
        }
    }
    impl MessageResponse for Response {
        fn messagetype_id() -> u32 {
            0
        }
    }
    impl ByteDataA for Message {
        fn as_byte_data(&self, vec: &mut Vec<u8>) {
            self.function_id.as_byte_data(vec);
            self.args.as_byte_data(vec);
        }
    }
    impl ByteData for Message {
        fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
        where
            R: std::io::Read,
        {
            Ok(Self {
                function_id: ByteData::from_byte_data(data)?,
                args: ByteData::from_byte_data(data)?,
            })
        }
    }
    impl ByteDataA for Response {
        fn as_byte_data(&self, vec: &mut Vec<u8>) {
            self.result.as_byte_data(vec);
        }
    }
    impl ByteData for Response {
        fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
        where
            R: std::io::Read,
        {
            Ok(Self {
                result: ByteData::from_byte_data(data)?,
            })
        }
    }
}

// implementations of ByteData for other data

type UsizeConstLen = u64;
type IsizeConstLen = i64;

impl ByteDataA for usize {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        (*self as UsizeConstLen).as_byte_data(vec)
    }
}
impl ByteData for usize {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok(UsizeConstLen::from_byte_data(data)? as _)
    }
}
impl ByteDataA for isize {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        (*self as IsizeConstLen).as_byte_data(vec)
    }
}
impl ByteData for isize {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok(IsizeConstLen::from_byte_data(data)? as _)
    }
}
impl ByteDataA for i32 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for i32 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 4];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for u32 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for u32 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 4];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for i64 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for i64 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 8];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for u64 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for u64 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 8];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for u128 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for u128 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 16];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for i128 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes())
    }
}
impl ByteData for i128 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 16];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for f64 {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_be_bytes());
    }
}
impl ByteData for f64 {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut b = [0u8; 8];
        data.read_exact(&mut b)?;
        Ok(Self::from_be_bytes(b))
    }
}
impl ByteDataA for String {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.len().as_byte_data(vec);
        vec.extend_from_slice(self.as_bytes());
    }
}
impl ByteData for String {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let len = ByteData::from_byte_data(data)?;
        let mut buf = vec![0; len];
        data.read_exact(buf.as_mut_slice());
        let str = String::from_utf8(buf).unwrap();
        Ok(str)
    }
}
impl<T> ByteDataA for &T
where
    T: ByteDataA,
{
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        (*self).as_byte_data(vec)
    }
}
impl<T> ByteDataA for Vec<T>
where
    T: ByteDataA,
{
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.len().as_byte_data(vec);
        for elem in self {
            elem.as_byte_data(vec);
        }
    }
}
impl<T> ByteData for Vec<T>
where
    T: ByteData,
{
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let len = usize::from_byte_data(data)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::from_byte_data(data)?);
        }
        Ok(vec)
    }
}
impl<A, B> ByteDataA for (A, B)
where
    A: ByteDataA,
    B: ByteDataA,
{
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.0.as_byte_data(vec);
        self.1.as_byte_data(vec);
    }
}
impl<A, B> ByteData for (A, B)
where
    A: ByteData,
    B: ByteData,
{
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok((
            ByteData::from_byte_data(data)?,
            ByteData::from_byte_data(data)?,
        ))
    }
}
impl<A, B, C, D, E> ByteDataA for (A, B, C, D, E)
where
    A: ByteDataA,
    B: ByteDataA,
    C: ByteDataA,
    D: ByteDataA,
    E: ByteDataA,
{
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.0.as_byte_data(vec);
        self.1.as_byte_data(vec);
        self.2.as_byte_data(vec);
        self.3.as_byte_data(vec);
        self.4.as_byte_data(vec);
    }
}
impl<A, B, C, D, E> ByteData for (A, B, C, D, E)
where
    A: ByteData,
    B: ByteData,
    C: ByteData,
    D: ByteData,
    E: ByteData,
{
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok((
            ByteData::from_byte_data(data)?,
            ByteData::from_byte_data(data)?,
            ByteData::from_byte_data(data)?,
            ByteData::from_byte_data(data)?,
            ByteData::from_byte_data(data)?,
        ))
    }
}
impl ByteDataA for VType {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.types.as_byte_data(vec)
    }
}
impl ByteData for VType {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok(Self {
            types: ByteData::from_byte_data(data)?,
        })
    }
}
impl ByteDataA for VSingleType {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        match self {
            Self::Bool => vec.push(b'b'),
            Self::Int => vec.push(b'i'),
            Self::Float => vec.push(b'f'),
            Self::String => vec.push(b'"'),
            Self::Tuple(v) => {
                vec.push(b't');
                v.as_byte_data(vec);
            }
            Self::List(v) => {
                vec.push(b'l');
                v.as_byte_data(vec);
            }
            Self::Function(f) => {
                vec.push(b'F');
                f.as_byte_data(vec);
            }
            Self::Thread(r) => {
                vec.push(b'T');
                r.as_byte_data(vec);
            }
            Self::Reference(r) => {
                vec.push(b'R');
                r.as_byte_data(vec);
            }
            Self::EnumVariant(e, v) => {
                vec.push(b'e');
                e.as_byte_data(vec);
                v.as_byte_data(vec);
            }
            Self::EnumVariantS(e, v) => {
                vec.push(b'E');
                e.as_byte_data(vec);
                v.as_byte_data(vec);
            }
            Self::CustomType(_) | Self::CustomTypeS(_) => {
                unreachable!("CustomType and CustomTypeS cannot be used in libraries [yet?].")
            }
        }
    }
}
impl ByteData for VSingleType {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut switch_byte = [0u8];
        data.read_exact(&mut switch_byte)?;
        Ok(match switch_byte[0] {
            b'b' => Self::Bool,
            b'i' => Self::Int,
            b'f' => Self::Float,
            b'"' => Self::String,
            b't' => Self::Tuple(ByteData::from_byte_data(data)?),
            b'l' => Self::List(ByteData::from_byte_data(data)?),
            b'F' => Self::Function(ByteData::from_byte_data(data)?),
            b'T' => Self::Thread(ByteData::from_byte_data(data)?),
            b'R' => Self::Reference(Box::new(ByteData::from_byte_data(data)?)),
            b'e' => Self::EnumVariant(
                ByteData::from_byte_data(data)?,
                ByteData::from_byte_data(data)?,
            ),
            b'E' => Self::EnumVariantS(
                ByteData::from_byte_data(data)?,
                ByteData::from_byte_data(data)?,
            ),
            _ => unreachable!("unexpected byte while reading single type"),
        })
    }
}
impl ByteDataA for VData {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        self.operate_on_data_immut(|v| v.as_byte_data(vec))
    }
}
impl ByteData for VData {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        Ok(VDataEnum::from_byte_data(data)?.to())
    }
}
impl ByteDataA for VDataEnum {
    fn as_byte_data(&self, vec: &mut Vec<u8>) {
        match self {
            Self::Bool(false) => vec.push(b'b'),
            Self::Bool(true) => vec.push(b'B'),
            Self::Int(num) => {
                vec.push(b'i');
                num.as_byte_data(vec);
            }
            Self::Float(num) => {
                vec.push(b'f');
                num.as_byte_data(vec);
            }
            Self::String(s) => {
                vec.push(b'"');
                s.as_byte_data(vec);
            }
            Self::Tuple(c) => {
                vec.push(b't');
                c.as_byte_data(vec);
            }
            Self::List(_, data) => {
                vec.push(b'l');
                data.as_byte_data(vec);
            }
            // TODO?
            Self::Function(_) => vec.push(b'F'),
            Self::Thread(..) => vec.push(b'T'),
            Self::Reference(r) => vec.push(b'R'),
            Self::EnumVariant(enum_id, inner) => {
                vec.push(b'E');
                enum_id.as_byte_data(vec);
                inner.as_byte_data(vec);
            }
        }
    }
}
impl ByteData for VDataEnum {
    fn from_byte_data<R>(data: &mut R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        let mut switch_byte = [0u8];
        data.read_exact(&mut switch_byte)?;
        Ok(match switch_byte[0] {
            b'b' => Self::Bool(false),
            b'B' => Self::Bool(true),
            b'i' => Self::Int(ByteData::from_byte_data(data)?),
            b'f' => Self::Float(ByteData::from_byte_data(data)?),
            b'"' => Self::String(ByteData::from_byte_data(data)?),
            b't' => Self::Tuple(ByteData::from_byte_data(data)?),
            b'l' => {
                let entries: Vec<VData> = ByteData::from_byte_data(data)?;
                Self::List(
                    entries.iter().fold(VType::empty(), |t, v| t | v.out()),
                    entries,
                )
            }
            b'E' => Self::EnumVariant(
                ByteData::from_byte_data(data)?,
                Box::new(ByteData::from_byte_data(data)?),
            ),
            _ => unreachable!("read invalid byte"),
        })
    }
}
