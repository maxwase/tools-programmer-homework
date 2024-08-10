use std::convert::Infallible;

use crate::BitWidth;

use super::{DisasmError, Disassembler};

/// RISC-V disassembler.
pub struct RiscV {
    width: BitWidth,
}

impl RiscV {
    /// Constructs a new [RiscV] disassembler, validating its options.
    pub fn new(width: BitWidth) -> Result<Self, DisasmError<Infallible>> {
        match width {
            BitWidth::Bit16 | BitWidth::Bit32 => Ok(Self { width }),
            unsupported => Err(DisasmError::WrongBitWidth(unsupported)),
        }
    }
}

impl Disassembler for RiscV {
    type Error = Infallible;

    fn disassemble<B: AsRef<[u8]>>(
        &self,
        _bytes: B,
        _options: &crate::format::AssemblerOutput,
    ) -> Result<Vec<String>, DisasmError<Self::Error>> {
        let _width = self.width;
        Err(DisasmError::Unimplemented)
    }
}
