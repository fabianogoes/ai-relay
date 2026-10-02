//! `relay-tui`: a read-only terminal observer for Relay workspaces.
//!
//! `core` is a pure port of `relay-core` (content in, state out, no disk).

pub mod app;
pub mod cli;
pub mod core;
pub mod theme;
pub mod view;
pub mod workspace;
