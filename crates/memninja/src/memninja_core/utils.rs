use crate::{MemType, MemValue, ScanType};
use hoodmem::scanner::{ScanFilter, Scanner};

#[derive(Debug)]
pub enum GenericScanFilter {
    U8(ScanFilter<u8>),
    U16(ScanFilter<u16>),
    U32(ScanFilter<u32>),
    U64(ScanFilter<u64>),
    I8(ScanFilter<i8>),
    I16(ScanFilter<i16>),
    I32(ScanFilter<i32>),
    I64(ScanFilter<i64>),
    F32(ScanFilter<f32>),
    F64(ScanFilter<f64>),
}

impl GenericScanFilter {
    /// Performs a new scan
    pub fn scan(&self, scanner: &mut Scanner) -> anyhow::Result<()> {
        match self {
            GenericScanFilter::U8(s) => scanner.scan(*s),
            GenericScanFilter::U16(s) => scanner.scan(*s),
            GenericScanFilter::U32(s) => scanner.scan(*s),
            GenericScanFilter::U64(s) => scanner.scan(*s),
            GenericScanFilter::I8(s) => scanner.scan(*s),
            GenericScanFilter::I16(s) => scanner.scan(*s),
            GenericScanFilter::I32(s) => scanner.scan(*s),
            GenericScanFilter::I64(s) => scanner.scan(*s),
            GenericScanFilter::F32(s) => scanner.scan(*s),
            GenericScanFilter::F64(s) => scanner.scan(*s),
        }
    }

    /// Create a scan filter for the given memory type, and optionally a value
    pub fn new(
        scan_type: ScanType,
        mem_type: MemType,
        mem_value: Option<MemValue>,
    ) -> anyhow::Result<Self> {
        match scan_type {
            ScanType::Exact => {
                if let Some(value) = mem_value {
                    Ok(match value {
                        MemValue::U8(v) => Self::U8(ScanFilter::Exact(v)),
                        MemValue::U16(v) => Self::U16(ScanFilter::Exact(v)),
                        MemValue::U32(v) => Self::U32(ScanFilter::Exact(v)),
                        MemValue::U64(v) => Self::U64(ScanFilter::Exact(v)),
                        MemValue::I8(v) => Self::I8(ScanFilter::Exact(v)),
                        MemValue::I16(v) => Self::I16(ScanFilter::Exact(v)),
                        MemValue::I32(v) => Self::I32(ScanFilter::Exact(v)),
                        MemValue::I64(v) => Self::I64(ScanFilter::Exact(v)),
                        MemValue::F32(v) => Self::F32(ScanFilter::Exact(v)),
                        MemValue::F64(v) => Self::F64(ScanFilter::Exact(v)),
                        MemValue::Null => anyhow::bail!("Cannot scan for unknown type"),
                    })
                } else {
                    anyhow::bail!("Cannot perform exact scan without a value");
                }
            }
            ScanType::Unknown => Ok(match mem_type {
                MemType::U8 => Self::U8(ScanFilter::Unknown::<u8>),
                MemType::U16 => Self::U16(ScanFilter::Unknown::<u16>),
                MemType::U32 => Self::U32(ScanFilter::Unknown::<u32>),
                MemType::U64 => Self::U64(ScanFilter::Unknown::<u64>),
                MemType::I8 => Self::I8(ScanFilter::Unknown::<i8>),
                MemType::I16 => Self::I16(ScanFilter::Unknown::<i16>),
                MemType::I32 => Self::I32(ScanFilter::Unknown::<i32>),
                MemType::I64 => Self::I64(ScanFilter::Unknown::<i64>),
                MemType::F32 => Self::F32(ScanFilter::Unknown::<f32>),
                MemType::F64 => Self::F64(ScanFilter::Unknown::<f64>),
                MemType::Unknown => anyhow::bail!("Cannot scan for unknown type"),
            }),
            ScanType::Increased => Ok(match mem_type {
                MemType::U8 => Self::U8(ScanFilter::Increased::<u8>),
                MemType::U16 => Self::U16(ScanFilter::Increased::<u16>),
                MemType::U32 => Self::U32(ScanFilter::Increased::<u32>),
                MemType::U64 => Self::U64(ScanFilter::Increased::<u64>),
                MemType::I8 => Self::I8(ScanFilter::Increased::<i8>),
                MemType::I16 => Self::I16(ScanFilter::Increased::<i16>),
                MemType::I32 => Self::I32(ScanFilter::Increased::<i32>),
                MemType::I64 => Self::I64(ScanFilter::Increased::<i64>),
                MemType::F32 => Self::F32(ScanFilter::Increased::<f32>),
                MemType::F64 => Self::F64(ScanFilter::Increased::<f64>),
                MemType::Unknown => anyhow::bail!("Cannot scan for Increased type"),
            }),
            ScanType::Decreased => Ok(match mem_type {
                MemType::U8 => Self::U8(ScanFilter::Decreased::<u8>),
                MemType::U16 => Self::U16(ScanFilter::Decreased::<u16>),
                MemType::U32 => Self::U32(ScanFilter::Decreased::<u32>),
                MemType::U64 => Self::U64(ScanFilter::Decreased::<u64>),
                MemType::I8 => Self::I8(ScanFilter::Decreased::<i8>),
                MemType::I16 => Self::I16(ScanFilter::Decreased::<i16>),
                MemType::I32 => Self::I32(ScanFilter::Decreased::<i32>),
                MemType::I64 => Self::I64(ScanFilter::Decreased::<i64>),
                MemType::F32 => Self::F32(ScanFilter::Decreased::<f32>),
                MemType::F64 => Self::F64(ScanFilter::Decreased::<f64>),
                MemType::Unknown => anyhow::bail!("Cannot scan for Decreased type"),
            }),
        }
    }
}
