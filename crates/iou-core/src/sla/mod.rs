//! SLA (Service Level Agreement) calculation module
//!
//! Provides deadline calculation with business hours, weekend skipping,
//! and holiday support.

mod calculator;

pub use calculator::{SlaCalculator, SlaConfig};
