// Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod evalue;
pub mod structured_claims;
pub mod topicid;

#[cfg(feature = "sim")]
pub mod boundary;
#[cfg(feature = "sim")]
pub mod experiments;
#[cfg(feature = "sim")]
pub mod labels;
#[cfg(feature = "sim")]
pub mod popper;
