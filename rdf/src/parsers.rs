// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! The trait a syntax extension implements, and the kit's running of it.
//!
//! An extension implements [`Syntax`]: a description of its syntax, and a function from a
//! document's bytes to the statements the document makes. [`respond`] does everything the
//! contract asks of a parser around that function, and [`export_syntax!`](crate::export_syntax)
//! calls it from the generated export.

use naamio_space_kit::components;

use crate::bindings::{wit_future, wit_stream};
use crate::contract::parser::{ParseError, ParseOptions};
use crate::contract::syntax::SyntaxDescription;
use crate::contract::terms::LocatedStatement;
use crate::wit_bindgen::{FutureReader, StreamReader};

/// The result of reading a document: every statement completed before the end of the
/// document or the first error, and the outcome the reading ended with.
///
/// The contract has a parser deliver every statement it completed before an error, so the
/// host can decide whether to keep them. An extension that stops at an error returns the
/// statements it has, together with the error.
#[derive(Debug)]
pub struct Reading {
	/// The statements, in document order.
	pub statements: Vec<LocatedStatement>,
	/// `Ok` if the whole document was read, or the error that stopped the reading.
	pub outcome: Result<(), ParseError>,
}

/// A syntax extension: one Resource Description Framework (RDF) syntax, and how to read it.
pub trait Syntax {
	/// Describes the syntax: its name, version, media types, file extensions, and which
	/// parts of RDF it can express. Hosts route documents to extensions by media type.
	fn description() -> SyntaxDescription;

	/// Reads `document`, given in the syntax's own encoding, into the statements it makes,
	/// as written.
	///
	/// `options` carries the base Internationalized Resource Identifier (IRI) and the media
	/// type the host gave. A syntax whose relative references the host resolves, as in the
	/// Turtle family, can ignore the base; a syntax that resolves IRIs itself uses it.
	fn read(document: &[u8], options: &ParseOptions) -> Reading;
}

/// Answers a host's call to `parse` for the syntax `S`.
///
/// Returns the statement stream and the outcome future at once, as the contract requires,
/// and then, after the export has returned, reads the whole document from its stream, reads
/// it with [`Syntax::read`], writes the statements, and resolves the future.
pub fn respond<S: Syntax>(
	document: StreamReader<u8>,
	options: ParseOptions,
) -> (
	StreamReader<LocatedStatement>,
	FutureReader<Result<(), ParseError>>,
) {
	let (statements, statements_reader) = wit_stream::new::<LocatedStatement>();
	let (outcome, outcome_reader) = wit_future::new::<Result<(), ParseError>>(|| Ok(()));

	components::after_return(async move {
		let bytes = document.collect().await;
		let reading = S::read(&bytes, &options);
		components::deliver(statements, reading.statements, outcome, reading.outcome).await;
	});

	(statements_reader, outcome_reader)
}
