#[derive(Debug, Clone)]
pub enum AttachTarget {
    Process(u32),
    Window(String),
    Other(String),
}

#[derive(Debug, Clone)]
pub enum AttachStatus {
    Detached,
    Attached(AttachTarget),
    Unknown,
}

impl Default for AttachStatus {
    fn default() -> Self {
        Self::Detached
    }
}


#[derive(Debug, Default, Clone)]
pub enum ScanStatus {
    /// Ready to scan
    #[default]
    Ready,
    /// A scan is currently in progress
    Scanning,
    /// Done scanning.
    Done(u64),
    /// Scan failed for some reason
    Failed(String),
    /// Unknown status
    Unknown
}


impl std::fmt::Display for ScanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanStatus::Ready => write!(f, "Ready to scan"),
            ScanStatus::Scanning => write!(f, "Scanning..."),
            ScanStatus::Done(num_results) => write!(f, "Scan complete ({} Results)", num_results),
            ScanStatus::Failed(reason) => write!(f, "Scan Failed ({})", reason),
            ScanStatus::Unknown => write!(f, ""),
        }
    }
}


#[derive(Debug, Default, PartialEq)]
pub enum AttachType {
    #[default]
    ByPID,
    ByWindowName,
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum ScanType {
    #[default]
    Exact,
    Unknown,
    Increased,
    Decreased,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fallback = format!("{:?}", self);
        write!(
            f,
            "{}",
            match self {
                ScanType::Exact => "Exact",
                ScanType::Unknown => "Unknown",
                _ => &fallback,
            }
        )
    }
}

pub enum MemValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Null,
}

impl std::fmt::Display for MemValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemValue::U8(x) => write!(f, "{}", x),
            MemValue::U16(x) => write!(f, "{}", x),
            MemValue::U32(x) => write!(f, "{}", x),
            MemValue::U64(x) => write!(f, "{}", x),
            MemValue::I8(x) => write!(f, "{}", x),
            MemValue::I16(x) => write!(f, "{}", x),
            MemValue::I32(x) => write!(f, "{}", x),
            MemValue::I64(x) => write!(f, "{}", x),
            MemValue::F32(x) => write!(f, "{}", x),
            MemValue::F64(x) => write!(f, "{}", x),
            MemValue::Null => write!(f, "null"),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum MemType {
    #[default]
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Unknown,
}

impl MemType {
    pub fn parse_value(&self, value: &str) -> anyhow::Result<MemValue> {
        Ok(match self {
            MemType::U8 => MemValue::U8(value.parse()?),
            MemType::U16 => MemValue::U8(value.parse()?),
            MemType::U32 => MemValue::U8(value.parse()?),
            MemType::U64 => MemValue::U8(value.parse()?),
            MemType::I8 => MemValue::U8(value.parse()?),
            MemType::I16 => MemValue::U8(value.parse()?),
            MemType::I32 => MemValue::U8(value.parse()?),
            MemType::I64 => MemValue::U8(value.parse()?),
            MemType::F32 => MemValue::U8(value.parse()?),
            MemType::F64 => MemValue::U8(value.parse()?),
            MemType::Unknown => anyhow::bail!("Cannot parse the unknown type"),
        })
    }
}

impl From<MemValue> for MemType {
    fn from(value: MemValue) -> Self {
        match value {
            MemValue::U8(_) => Self::U8,
            MemValue::U16(_) => Self::U16,
            MemValue::U32(_) => Self::U32,
            MemValue::U64(_) => Self::U64,
            MemValue::I8(_) => Self::I8,
            MemValue::I16(_) => Self::I16,
            MemValue::I32(_) => Self::I32,
            MemValue::I64(_) => Self::I64,
            MemValue::F32(_) => Self::F32,
            MemValue::F64(_) => Self::F64,
            MemValue::Null => Self::Unknown,
        }
    }
}

impl std::fmt::Display for MemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MemType::U8 => "8-bit Integer (unsigned)",
                MemType::U16 => "16-bit Integer (unsigned)",
                MemType::U32 => "32-bit Integer (unsigned)",
                MemType::U64 => "64-bit Integer (unsigned)",
                MemType::I8 => "8-bit Integer (signed)",
                MemType::I16 => "16-bit Integer (signed)",
                MemType::I32 => "32-bit Integer (signed)",
                MemType::I64 => "64-bit Integer (signed)",
                MemType::F32 => "Float (32-bit)",
                MemType::F64 => "Float (64-bit)",
                MemType::Unknown => "Unknown",
            }
        )
    }
}

pub enum CheatType {
    Simple { addr: u64, mem_type: MemType },
}

pub trait CheatSummary {
    fn get_summary(&self) -> String;
}

impl CheatSummary for CheatType {
    fn get_summary(&self) -> String {
        match self {
            CheatType::Simple { addr, mem_type } => format!("[{}] 0x{:016x}", mem_type, addr),
        }
    }
}

impl std::fmt::Display for CheatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheatType::Simple { addr, mem_type } => write!(f, "Simple ({})", mem_type),
        }
    }
}

pub struct Cheat {
    pub enabled: bool,
    pub name: String,
    pub cheat_type: CheatType,
}

impl CheatSummary for Cheat {
    fn get_summary(&self) -> String {
        self.cheat_type.get_summary()
    }
}
