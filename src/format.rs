use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Output disassembly formatting options.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AssemblerOutput {
    /// Show addresses.
    address: ShowAddress,
    /// Stop at address.
    stop_at: Option<usize>,
    /// Show instructions in the upper case.
    upper_case: bool,
    /// Show how many cycles does the instruction take.
    cycles: bool,
    /// Names for symbols.
    // The conversion is needed due to JSON standard. En enum discriminant is converted to a string.
    #[serde_as(as = "Option<HashMap<serde_with::json::JsonString, _>>")]
    symbol_table: Option<HashMap<SymbolInfo, String>>,
}

impl Default for AssemblerOutput {
    fn default() -> Self {
        Self {
            address: Default::default(),
            upper_case: true,
            cycles: false,
            symbol_table: Default::default(),
            stop_at: None,
        }
    }
}

/// Addresses options.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum ShowAddress {
    /// Show addresses with a provided offset.
    Start(usize),
    /// Do not show addresses.
    None,
}

impl Default for ShowAddress {
    fn default() -> Self {
        Self::Start(0)
    }
}

/// A symbol attributes.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy)]
pub struct SymbolInfo {
    address: usize,
    scope: Scope,
    // TODO: more symbol attributes
}

impl SymbolInfo {
    /// Constructs a new symbol.
    pub fn new(address: usize, scope: Scope) -> Self {
        Self { address, scope }
    }
}

/// Defines a symbol scope.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy)]
pub enum Scope {
    Local,
    Global,
    // TODO: more granular scopes
}

impl AssemblerOutput {
    /// Show addresses in a disassembly output.
    pub fn with_addresses(mut self, address: ShowAddress) -> Self {
        self.address = address;
        self
    }

    /// Choose case of a disassembly output.
    pub fn with_upper_case(mut self, capital: bool) -> Self {
        self.upper_case = capital;
        self
    }

    /// Show cycles in a disassembly output.
    pub fn with_cycles(mut self, cycles: bool) -> Self {
        self.cycles = cycles;
        self
    }

    /// Replace [SymbolInfo] with a name in a disassembly output.
    pub fn with_symbol_table(mut self, table: HashMap<SymbolInfo, String>) -> Self {
        self.symbol_table = Some(table);
        self
    }

    /// Which address to stop at?
    pub fn with_stop(mut self, stop: usize) -> Self {
        self.stop_at = Some(stop);
        self
    }

    /// Show address?
    pub fn address(&self) -> &ShowAddress {
        &self.address
    }

    /// Which address to stop at?
    pub fn stop_at(&self) -> Option<usize> {
        self.stop_at
    }

    /// Use upper case?
    pub fn upper_case(&self) -> bool {
        self.upper_case
    }

    /// Show cycles?
    pub fn cycles(&self) -> bool {
        self.cycles
    }

    /// Use symbol table?
    pub fn symbol_table(&self) -> Option<&HashMap<SymbolInfo, String>> {
        self.symbol_table.as_ref()
    }
}
