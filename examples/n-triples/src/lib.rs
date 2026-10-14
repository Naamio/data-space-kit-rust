// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! An example syntax extension that reads N-Triples, written with the Rust kit.
//!
//! N-Triples writes one triple per line, with every Internationalized Resource Identifier
//! (IRI) absolute and no prefixes, which makes it the smallest complete example of a syntax
//! extension. Everything here apart from [`grammar`] is the extension's whole connection to
//! Naamio Space: a description of the syntax, and one line exporting it.

pub mod grammar;

use naamio_space_kit_rdf::prelude::*;

/// The N-Triples syntax, as an extension.
#[derive(Debug)]
pub struct NTriples;

impl Syntax for NTriples {
	/// N-Triples 1.2, registered as `application/n-triples`. It cannot express named graphs;
	/// N-Quads is the syntax that adds them. This example does not read RDF 1.2 triple terms,
	/// so it does not claim them.
	fn description() -> SyntaxDescription {
		SyntaxDescription {
			name: "N-Triples".to_owned(),
			version: "1.2".to_owned(),
			media_types: vec!["application/n-triples".to_owned()],
			file_extensions: vec!["nt".to_owned()],
			features: SyntaxFeatures::BASE_DIRECTION,
		}
	}

	/// Reads the document line by line. N-Triples has no relative IRIs, so the base in the
	/// options is never needed.
	fn read(document: &[u8], _options: &ParseOptions) -> Reading {
		grammar::read(document)
	}
}

export_syntax!(NTriples);
