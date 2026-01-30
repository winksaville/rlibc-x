//! High-level build options that expand to lower-level flags.
//!
//! These provide "easy generality" - user picks intent, implementation
//! details (which cargo/rustc flags to set) are handled automatically.

use serde::{Deserialize, Serialize};

/// Panic handling strategy.
///
/// This is a high-level option that automatically sets the appropriate
/// cargo and rustc flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PanicMode {
    /// Default Rust behavior - panic unwinds the stack.
    #[default]
    Unwind,

    /// Panic aborts immediately (no unwinding).
    /// Sets: rustc -C panic=abort
    Abort,

    /// Panic aborts with no formatting overhead (nightly only).
    /// Sets: cargo -Z panic-immediate-abort AND rustc -C panic=immediate-abort
    /// This eliminates all panic formatting machinery for smallest binaries.
    ImmediateAbort,
}

impl PanicMode {
    /// Returns true if this mode requires nightly toolchain.
    pub fn requires_nightly(&self) -> bool {
        matches!(self, PanicMode::ImmediateAbort)
    }

    /// Returns the cargo -Z flag if needed.
    pub fn cargo_z_flag(&self) -> Option<&'static str> {
        match self {
            PanicMode::ImmediateAbort => Some("panic-immediate-abort"),
            _ => None,
        }
    }

    /// Returns the rustc -C panic= value.
    pub fn rustc_panic_value(&self) -> Option<&'static str> {
        match self {
            PanicMode::Unwind => None, // default, no flag needed
            PanicMode::Abort => Some("abort"),
            PanicMode::ImmediateAbort => Some("immediate-abort"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwind_is_default() {
        assert_eq!(PanicMode::default(), PanicMode::Unwind);
    }

    #[test]
    fn only_immediate_abort_requires_nightly() {
        assert!(!PanicMode::Unwind.requires_nightly());
        assert!(!PanicMode::Abort.requires_nightly());
        assert!(PanicMode::ImmediateAbort.requires_nightly());
    }

    #[test]
    fn cargo_z_flag_only_for_immediate_abort() {
        assert_eq!(PanicMode::Unwind.cargo_z_flag(), None);
        assert_eq!(PanicMode::Abort.cargo_z_flag(), None);
        assert_eq!(
            PanicMode::ImmediateAbort.cargo_z_flag(),
            Some("panic-immediate-abort")
        );
    }

    #[test]
    fn rustc_panic_values() {
        assert_eq!(PanicMode::Unwind.rustc_panic_value(), None);
        assert_eq!(PanicMode::Abort.rustc_panic_value(), Some("abort"));
        assert_eq!(
            PanicMode::ImmediateAbort.rustc_panic_value(),
            Some("immediate-abort")
        );
    }
}
