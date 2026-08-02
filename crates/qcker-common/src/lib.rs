//! Qcker common utilities and error types.
//!
//! This crate provides shared types used across the Qcker workspace,
//! including the [`QckerError`] type with structured error codes,
//! source location tracking, and retryable error detection.

#![allow(clippy::result_large_err)]

pub mod error;
pub mod fs;
pub mod hash;
pub mod id;
pub mod tar;

pub use error::{QckerError, Result};
