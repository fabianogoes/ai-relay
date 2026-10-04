//! `relay-tui`: a read-only terminal observer for Relay workspaces.
//!
//! `core` is pure (content in, state out, no disk).

pub mod app;
pub mod cli;
pub mod core;
pub mod language;
pub mod nav;
pub mod suggest;
pub mod theme;
pub mod view;
pub mod workspace;
