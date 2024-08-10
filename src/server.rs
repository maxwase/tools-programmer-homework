use std::str::FromStr;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use disassembler::{mos6502, risc_v, x86, AssemblerOutput, BitWidth, DisasmError, Disassembler};

pub const X86_ENDPOINT: &str = "/x86";
pub const MOS6502_ENDPOINT: &str = "/mos6502";
pub const RISC_V_ENDPOINT: &str = "/risc_v";

/// Common input to the disassembly service.
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    bytes: Vec<u8>,
    width: BitWidth,
    syntax: Option<String>,
    format: AssemblerOutput,
}

impl Payload {
    /// Gets requested bytes.
    fn bytes(&self) -> &[u8] {
        // With this method it's possible to add logic like this in future is needed
        /*
        let start = self.start.unwrap_or(0);
        let stop = self.start.unwrap_or_else(|| self.data.len());
        let range = start..stop;
         */
        &self.bytes
    }
}

pub async fn handle_mos6502(
    Json(payload): Json<Payload>,
) -> Result<Response, DisasmError<<mos6502::Mos6502 as Disassembler>::Error>> {
    let disasm = mos6502::Mos6502;
    let res = disasm.disassemble(payload.bytes(), &payload.format)?;

    Ok(Json(res).into_response())
}

pub async fn handle_risc_v(
    Json(payload): Json<Payload>,
) -> Result<Response, DisasmError<<risc_v::RiscV as Disassembler>::Error>> {
    let disasm = &risc_v::RiscV::new(payload.width)?;
    let res = disasm.disassemble(payload.bytes(), &payload.format)?;

    Ok(Json(res).into_response())
}

pub async fn handle_x86(
    Json(payload): Json<Payload>,
) -> Result<Response, DisasmError<<x86::X86 as Disassembler>::Error>> {
    let syntax = match &payload.syntax {
        Some(requested) => x86::Syntax::from_str(requested)?,
        None => x86::Syntax::default(),
    };

    let disasm = x86::X86::new(syntax, payload.width)?;
    let res = disasm.disassemble(payload.bytes(), &payload.format)?;

    Ok(Json(res).into_response())
}

/// These are integration tests, however it's hard to move them to root/test
/// due to cargo's inability to use `bin`'s symbols over there.
///
/// **These tests expect to find a running server on [tests::URL].**
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use disassembler::*;
    use reqwest::StatusCode;

    use super::*;

    // I think that unit test should not depend on external system state when they can,
    // that's why I prefer to start the server here once.
    // For the sake of simplicity, I decided to leave the requirement to run the server first.
    // static SERVER_STARTED: AtomicBool = AtomicBool::new(false);

    const MOS6502_TEST_BYTES: &[u8] = &[0xa9, 0xbd, 0xa0, 0xbd, 0x20, 0x28, 0xba];

    fn url(endpoint: &str) -> String {
        format!("http://localhost:9999{endpoint}")
    }

    #[tokio::test]
    async fn test_mos6502() {
        let expected = [
            "0000 A9 BD    LDA #$BD",
            "0002 A0 BD    LDY #$BD",
            "0004 20 28 BA JSR $BA28",
        ];

        assert_eq!(
            expected.as_slice(),
            test_mos6502_impl(MOS6502_TEST_BYTES).await
        );

        let bytes = tokio::fs::read("test-bin/mos6502/test1.bin").await.unwrap();

        let expected = tokio::fs::read_to_string("test-bin/mos6502/test1.out")
            .await
            .unwrap();
        let expected = expected.lines().collect::<Vec<_>>();

        assert_eq!(expected, test_mos6502_impl(&bytes).await);

        let bytes = tokio::fs::read("test-bin/mos6502/test2.bin").await.unwrap();

        let expected = tokio::fs::read_to_string("test-bin/mos6502/test2.out")
            .await
            .unwrap();
        let expected = expected.lines().collect::<Vec<_>>();

        assert_eq!(expected.as_slice(), test_mos6502_impl(&bytes).await);
    }

    async fn test_mos6502_impl(bytes: &[u8]) -> Vec<String> {
        let client = reqwest::Client::builder().build().unwrap();

        let payload = Payload {
            bytes: bytes.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default(),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        resp.json().await.unwrap()
    }

    #[tokio::test]
    async fn test_mos6502_offset() {
        let client = reqwest::Client::new();
        let payload = Payload {
            bytes: MOS6502_TEST_BYTES.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default().with_addresses(ShowAddress::Start(0xA)),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = [
            "000A A9 BD    LDA #$BD",
            "000C A0 BD    LDY #$BD",
            "000E 20 28 BA JSR $BA28",
        ];
        assert_eq!(expected.as_slice(), resp);
    }

    #[tokio::test]
    async fn test_mos6502_byte_offset() {
        let client = reqwest::Client::new();
        let payload = Payload {
            bytes: MOS6502_TEST_BYTES.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default().with_addresses(ShowAddress::Start(0xA)),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = [
            "000A A9 BD    LDA #$BD",
            "000C A0 BD    LDY #$BD",
            "000E 20 28 BA JSR $BA28",
        ];
        assert_eq!(expected.as_slice(), resp);
    }

    #[tokio::test]
    async fn test_mos6502_no_address() {
        let client = reqwest::Client::new();
        let payload = Payload {
            bytes: MOS6502_TEST_BYTES.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default().with_addresses(ShowAddress::None),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = ["LDA #$BD", "LDY #$BD", "JSR $BA28"];
        assert_eq!(expected.as_slice(), resp);
    }

    #[tokio::test]
    async fn test_mos6502_no_address_lowercase() {
        let client = reqwest::Client::new();
        let payload = Payload {
            bytes: MOS6502_TEST_BYTES.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default()
                .with_addresses(ShowAddress::None)
                .with_upper_case(false),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = ["lda #$bd", "ldy #$bd", "jsr $ba28"];
        assert_eq!(expected.as_slice(), resp);
    }

    #[tokio::test]
    async fn test_mos6502_unsupported() {
        let client = reqwest::Client::new();
        let payload = Payload {
            bytes: MOS6502_TEST_BYTES.to_vec(),
            width: BitWidth::Bit8,
            format: AssemblerOutput::default()
                .with_cycles(true)
                .with_symbol_table(HashMap::from([(
                    SymbolInfo::new(0xBA28, Scope::Global),
                    "SUBROUTINE".to_string(),
                )])),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();

        assert_eq!(
            resp.error_for_status_ref().unwrap_err().status().unwrap(),
            StatusCode::NOT_IMPLEMENTED
        );

        let error: String = resp.json().await.unwrap();
        assert_eq!("Unsupported disassembler option", error);
    }

    #[tokio::test]
    async fn test_mos6502_with_x86() {
        let client = reqwest::Client::new();
        let bytes = std::fs::read("test-bin/x86/test.bin").unwrap();

        let payload = Payload {
            bytes,
            width: BitWidth::Bit8,
            format: AssemblerOutput::default().with_stop(0xA),
            syntax: None,
        };

        let url = url(MOS6502_ENDPOINT);
        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = [
            "0000 7F",
            "0001 45 4C    EOR $4C",
            "0003 46 02    LSR $02",
            "0005 01 01    ORA ($01,X)",
            "0007 00       BRK",
            "0008 00       BRK",
            "0009 00       BRK",
            "000A 00       BRK",
        ];
        assert_eq!(expected.as_slice(), resp)
    }

    #[tokio::test]
    async fn test_x86() {
        let client = reqwest::Client::new();
        let bytes = tokio::fs::read("test-bin/x86/test.bin").await.unwrap();

        let mut payload = Payload {
            bytes,
            width: BitWidth::Bit64,
            format: AssemblerOutput::default().with_stop(0xA),
            syntax: Some("att".to_string()),
        };

        let url = url(X86_ENDPOINT);
        let resp = client.post(&url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = [
            "0x00000000 JG 0x0000000000000047",
            "0x00000002 ADD (%RCX),%R8B",
            "0x00000006 ADD %EAX,(%RAX)",
            "0x00000008 ADD %AL,(%RAX)",
        ];
        assert_eq!(expected.as_slice(), resp);

        payload.syntax = None;

        let resp = client.post(url).json(&payload).send().await.unwrap();
        let resp: Vec<String> = resp.json().await.unwrap();

        let expected = [
            "0x00000000 JG SHORT 0000000000000047h",
            "0x00000002 ADD R8B,[RCX]",
            "0x00000006 ADD [RAX],EAX",
            "0x00000008 ADD [RAX],AL",
        ];
        assert_eq!(expected.as_slice(), resp)
    }

    #[tokio::test]
    async fn test_unimplemented() {
        let client = reqwest::Client::new();
        let bytes = tokio::fs::read("test-bin/x86/test.bin").await.unwrap();

        let payload = Payload {
            bytes,
            width: BitWidth::Bit16,
            format: AssemblerOutput::default(),
            syntax: None,
        };

        let url = url(RISC_V_ENDPOINT);
        let resp = client.post(&url).json(&payload).send().await.unwrap();

        assert_eq!(
            resp.error_for_status_ref().unwrap_err().status().unwrap(),
            StatusCode::NOT_IMPLEMENTED
        );

        let error: String = resp.json().await.unwrap();
        assert_eq!("The implementation has not been done", error);
    }
}
