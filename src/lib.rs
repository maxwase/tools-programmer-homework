use std::fmt;

use serde::{Deserialize, Serialize};

mod disasm;
mod format;

pub use {disasm::*, format::*};

/// Architecture bit width.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum BitWidth {
    Bit8 = 8,
    Bit16 = 16,
    Bit32 = 32,
    Bit64 = 64,
}

impl fmt::Display for BitWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitWidth::Bit8 => "8 bit",
            BitWidth::Bit16 => "16 bit",
            BitWidth::Bit32 => "32 bit",
            BitWidth::Bit64 => "64 bit",
        }
        .fmt(f)
    }
}
