// Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors
// SPDX-License-Identifier: Apache-2.0

//! discos-core
//!
//! DiscOS is the untrusted discovery/userland side of the system.
//!
//! This crate contains *simulation-grade* discovery algorithms and system tests
//! (e.g., probing attacks) used to validate EvidenceOS defenses.

pub mod boundary;
pub mod labels;

pub mod popper;
pub mod structured_claims;
pub mod topicid;
