//! Platform-independent core: data model, layout normalization, rule engine, persistence.
//! This crate must never depend on Windows APIs so it can be unit-tested anywhere.

pub mod brand;
pub mod config_store;
pub mod date_group;
pub mod geometry;
pub mod i18n;
pub mod model;
pub mod rules;

pub use config_store::{ConfigStore, LoadOutcome};
pub use date_group::{CivilDate, DateBucket, date_bucket};
pub use model::*;
pub use rules::{Cond, Decision, ItemFacts, Rule, RuleSet, StrOp, Target, TypeCategory};
