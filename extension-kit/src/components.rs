// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! Mechanics for extensions delivered as WebAssembly components.
//!
//! A streaming export, such as a parser's `parse`, returns a stream and a future before it
//! has done its work, and does the work afterwards in a task it spawns. The task runs within
//! the export's own component-model task, which is why such exports are asynchronous in
//! their contracts. These functions are that pattern, written once.

use std::future::Future;

use wit_bindgen::{FutureWriter, StreamWriter};

/// Runs `work` after the current export returns, within the export's task.
///
/// The export that calls this must be asynchronous in its contract. A synchronous export has
/// no task left for the work to run in once it returns, and the work never runs.
pub fn after_return(work: impl Future<Output = ()> + 'static) {
	wit_bindgen::spawn_local(work);
}

/// Writes every item to `stream`, closes it, and then resolves `future` with `outcome`.
///
/// The order is the contract's: a host that reads the outcome knows every item has already
/// been written. If the host stops reading the stream, the items it did not read are
/// discarded, because a host stops only when it wants no more. If it has stopped listening
/// for the outcome as well, there is no one left to tell, and the outcome is discarded too.
pub async fn deliver<Item, Outcome>(
	mut stream: StreamWriter<Item>,
	items: Vec<Item>,
	future: FutureWriter<Outcome>,
	outcome: Outcome,
) where
	Item: 'static,
	Outcome: 'static,
{
	// `write_all` hands back the items the host did not read. The host stopped reading
	// because it wanted no more, so they are dropped.
	let _ = stream.write_all(items).await;
	drop(stream);
	// The write fails only when the host has stopped listening for the outcome, and then
	// there is no one to give it to.
	let _ = future.write(outcome).await;
}
