//! Different disassemblers for different architectures.

use std::error::Error as StdError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::{format::AssemblerOutput, BitWidth};

pub mod mos6502;
pub mod risc_v;
pub mod x86;

/// A general disassembler architecture endpoint error.
#[derive(Error, Debug)]
pub enum DisasmError<ArchError: StdError> {
    #[error("Unsupported disassembler option")]
    UnsupportedOption,
    #[error("The implementation has not been done")]
    Unimplemented,
    #[error("Missing disassembler option")]
    MissingInfo,
    #[error("Invalid architecture bit width: {0}")]
    WrongBitWidth(BitWidth),
    #[error(transparent)]
    Arch(#[from] ArchError),
}

impl<E: StdError + IntoResponse> IntoResponse for DisasmError<E> {
    fn into_response(self) -> Response {
        let code = match self {
            Self::UnsupportedOption | Self::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Self::WrongBitWidth(_) | Self::MissingInfo => StatusCode::BAD_REQUEST,
            Self::Arch(e) => return e.into_response(),
        };

        (code, Json(self.to_string())).into_response()
    }
}

// This could have been a struct with an enum `Arch`,
// however instead of that I chose to do it as a trait for easier external extension.

// It would be better to return a wrapper over abstract ASM instructions that would
// accept [AssemblerOutput] when formatted, however it is too complicated for this assignment.

/// An abstraction over different disassemblers.
pub trait Disassembler {
    /// A specific disassembler error.
    type Error: StdError;

    /// Performs a disassembly operation on `bytes` with given `options`.
    fn disassemble<B: AsRef<[u8]>>(
        &self,
        bytes: B,
        format: &AssemblerOutput,
    ) -> Result<Vec<String>, DisasmError<Self::Error>>;
}
