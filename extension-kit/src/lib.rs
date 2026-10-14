// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! The foundation every kind of Naamio Space extension builds on, whichever way it is
//! delivered.
//!
//! The crate for each kind of extension, such as `naamio-space-kit-rdf`, builds on
//! this one. It holds the mechanics those crates share, so that each kind's crate holds only
//! what is specific to its contract. The mechanics are grouped by the way an extension is
//! delivered: [`components`] for WebAssembly components, which is how the extension host
//! runs extensions today.
//!
//! [`wit_bindgen`] is re-exported so that every crate in a component uses the same version
//! of it. Tasks spawned after an export returns run only when the bindings and the runtime
//! come from one version.

pub mod components;

pub use wit_bindgen;
