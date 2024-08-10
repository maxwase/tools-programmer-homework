use std::str::FromStr;

use axum::response::IntoResponse;
use iced_x86::{Decoder, DecoderOptions, Formatter, GasFormatter, Instruction, IntelFormatter};
use thiserror::Error;

use crate::{format::AssemblerOutput, BitWidth, ShowAddress};

use super::{DisasmError, Disassembler};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Iced disassembler error")]
    Iced(#[source] iced_x86::IcedError), // not in use, PoC
    #[error("Unsupported syntax: {0}")]
    UnsupportedSyntax(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.to_string().into_response()
    }
}

/// x86 disassembler.
pub struct X86 {
    syntax: Syntax,
    width: BitWidth,
}

/// Output disassembly syntax.
#[derive(Debug, Default, PartialEq)]
pub enum Syntax {
    #[default]
    #[doc(alias = "XSS")]
    Intel,
    #[doc(alias = "Gas")]
    Att,
}

impl FromStr for Syntax {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "intel" => Ok(Self::Intel),
            "att" | "at&t" => Ok(Self::Att),
            _ => Err(Error::UnsupportedSyntax(s.to_string())),
        }
    }
}

impl X86 {
    /// Constructs a new [X86] disassembler, validating its options.
    pub fn new(syntax: Syntax, width: BitWidth) -> Result<Self, DisasmError<Error>> {
        match width {
            BitWidth::Bit16 | BitWidth::Bit32 | BitWidth::Bit64 => Ok(Self { syntax, width }),
            unsupported => Err(DisasmError::WrongBitWidth(unsupported)),
        }
    }
}

impl Disassembler for X86 {
    type Error = Error;

    fn disassemble<B: AsRef<[u8]>>(
        &self,
        bytes: B,
        options: &AssemblerOutput,
    ) -> Result<Vec<String>, DisasmError<Error>> {
        if options.symbol_table().is_some() || options.cycles() {
            return Err(DisasmError::UnsupportedOption);
        }

        let mut decoder = Decoder::new(
            self.width as u8 as u32,
            bytes.as_ref(),
            DecoderOptions::NONE,
        );

        let formatter = match self.syntax {
            Syntax::Intel => &mut IntelFormatter::new() as &mut dyn Formatter,
            Syntax::Att => &mut GasFormatter::new() as &mut dyn Formatter,
        };

        formatter
            .options_mut()
            .set_uppercase_all(options.upper_case());

        let mut res = vec![];
        let mut output = String::new();
        let mut instruction = Instruction::default();

        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);

            if options
                .stop_at()
                .is_some_and(|stop| instruction.ip() >= stop as u64)
            {
                break;
            }

            output.clear();
            formatter.format(&instruction, &mut output);

            match *options.address() {
                ShowAddress::Start(offset) => {
                    let ip = instruction.ip() + offset as u64;

                    let line = if options.upper_case() {
                        format!("0x{ip:08X} {output}")
                    } else {
                        format!("0x{ip:08x} {output}")
                    };
                    res.push(line)
                }
                ShowAddress::None => res.push(output.clone()),
            };
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_stop() {
        let bytes = fs::read("test-bin/x86/test.bin").unwrap();
        let output = X86::new(Syntax::Intel, BitWidth::Bit64)
            .unwrap()
            .disassemble(
                &bytes,
                &AssemblerOutput::default()
                    .with_stop(10)
                    .with_addresses(ShowAddress::None),
            )
            .unwrap();

        assert_eq!(
            output,
            [
                "JG SHORT 0000000000000047h",
                "ADD R8B,[RCX]",
                "ADD [RAX],EAX",
                "ADD [RAX],AL"
            ]
        );
    }

    #[test]
    fn test_offset() {
        let bytes = fs::read("test-bin/x86/test.bin").unwrap();
        let output = X86::new(Syntax::Intel, BitWidth::Bit64)
            .unwrap()
            .disassemble(
                &bytes,
                &AssemblerOutput::default()
                    .with_addresses(ShowAddress::Start(0xFFF))
                    .with_stop(10),
            )
            .unwrap();

        assert_eq!(
            output,
            [
                "0x00000FFF JG SHORT 0000000000000047h",
                "0x00001001 ADD R8B,[RCX]",
                "0x00001005 ADD [RAX],EAX",
                "0x00001007 ADD [RAX],AL"
            ]
        );
    }
}
