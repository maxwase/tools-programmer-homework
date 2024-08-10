use std::convert::Infallible;

use crate::{
    disasm::DisasmError,
    format::{AssemblerOutput, ShowAddress},
};

use super::Disassembler;

/// MOS6502 disassembler.
pub struct Mos6502;

impl Disassembler for Mos6502 {
    /// [rs6502] crate does not have an `Error`.
    type Error = Infallible;

    fn disassemble<B: AsRef<[u8]>>(
        &self,
        bytes: B,
        options: &AssemblerOutput,
    ) -> Result<Vec<String>, DisasmError<Infallible>> {
        if options.symbol_table().is_some() || options.cycles() {
            return Err(DisasmError::UnsupportedOption);
        }

        let disasm = match *options.address() {
            ShowAddress::None => rs6502::Disassembler::with_code_only(),
            ShowAddress::Start(offset) => rs6502::Disassembler::with_offset(offset as u16),
        };

        let disasm = disasm.disassemble_with_addresses(bytes.as_ref());

        let processed = disasm
            .into_iter()
            .take_while(|(_, addr)| match options.stop_at() {
                Some(stop) => usize::from(*addr) <= stop,
                None => true,
            })
            .map(|(mut line, _)| {
                // strip `\n` added by the library
                line.pop();

                if options.upper_case() {
                    line
                } else {
                    line.to_ascii_lowercase()
                }

                // TODO: handle symbol map
            });

        Ok(processed.collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop() {
        let output = Mos6502
            .disassemble(
                &[0xa9, 0xbd, 0xa0, 0xbd, 0x20, 0x28, 0xba],
                &AssemblerOutput::default().with_stop(2),
            )
            .unwrap();

        assert_eq!(output, ["0000 A9 BD    LDA #$BD", "0002 A0 BD    LDY #$BD"]);
    }
}
