// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! Writing Naamio Space extensions that read Resource Description Framework (RDF) syntaxes.
//!
//! A syntax extension implements the `linked-data:rdf` contract. It reads one syntax and
//! reports the statements a document makes, as written; Naamio Space resolves, checks and
//! stores what they mean. With this crate, an extension is a type that implements
//! [`Syntax`], exported with [`export_syntax!`]:
//!
//! ```ignore
//! use naamio_space_kit_rdf::prelude::*;
//!
//! struct MySyntax;
//!
//! impl Syntax for MySyntax {
//!     fn description() -> SyntaxDescription { /* name, media types, features */ }
//!     fn read(document: &[u8], options: &ParseOptions) -> Reading { /* the grammar */ }
//! }
//!
//! export_syntax!(MySyntax);
//! ```
//!
//! The crate takes care of the rest: the bindings, the asynchronous export, reading the
//! document from its stream, and writing the statements and the outcome after the export
//! has returned.
//!
//! - [`prelude`] holds everything an extension usually needs, for a single glob import.
//! - [`contract`] holds the contract's types, by the interface that declares them.
//! - [`statements`] has builders for the contract's terms and statements.

// Lets the generated bindings name this crate's re-export of `wit_bindgen` by the same path
// from inside this crate as from an extension's crate.
extern crate self as naamio_space_kit_rdf;

pub mod parsers;
pub mod statements;

pub use naamio_space_kit::wit_bindgen;
pub use parsers::{Reading, Syntax};

/// The `linked-data:rdf` contract's types, by the interface that declares them.
///
/// The paths match the contract's WebAssembly Interface Type (WIT) files: `terms` for terms
/// and statements, `syntax` for describing a syntax, and `parser` for what a parser reports.
/// They lead to the generated [`bindings`], without the generator's own module structure.
pub mod contract {
	pub use crate::bindings::exports::linked_data::rdf::parser;
	pub use crate::bindings::linked_data::rdf::{syntax, terms};
}

/// Everything a syntax extension usually needs, for `use naamio_space_kit_rdf::prelude::*;`.
///
/// It holds the [`Syntax`](crate::Syntax) trait and [`Reading`](crate::Reading), the
/// [`export_syntax!`](crate::export_syntax) macro, the [`statements`](crate::statements)
/// builders, and the contract types a grammar names.
pub mod prelude {
	pub use crate::contract::parser::{ParseError, ParseOptions, SyntaxError};
	pub use crate::contract::syntax::{SyntaxDescription, SyntaxFeatures};
	pub use crate::contract::terms::{
		Annotation, BaseDirection, BlankNode, IriReference, Language, Literal, LocatedStatement,
		Node, PrefixDeclaration, PrefixedName, Quad, Span, Statement, Term, Triple,
	};
	pub use crate::export_syntax;
	pub use crate::statements::*;
	pub use crate::{Reading, Syntax};
}

/// Rust bindings for the `parser-component` world of the `linked-data:rdf` contract.
///
/// wit-bindgen generates them from the WebAssembly Interface Type (WIT) files in the
/// `specifications/wasi` submodule. They are public so that extensions can name the
/// contract's types, and their export macro is public so that [`export_syntax!`] can export
/// an extension from the extension's own crate.
///
/// Some of the crate's lints reach the code wit-bindgen generates. The exemptions below cover
/// that generated code only.
#[expect(
	clippy::same_length_and_capacity,
	reason = "wit-bindgen's generated code rebuilds vectors passed across the boundary"
)]
#[expect(
	missing_docs,
	reason = "wit-bindgen documents the types the contract documents, but not the modules, \
		aliases and traits it adds around them"
)]
pub mod bindings {
	crate::wit_bindgen::generate!({
		path: "../specifications/wasi/rdf",
		world: "parser-component",
		runtime_path: "::naamio_space_kit_rdf::wit_bindgen::rt",
		pub_export_macro: true,
		additional_derives: [PartialEq, Eq],
	});
}

/// Exports a type that implements [`Syntax`] as the extension's `parser`.
///
/// Use it once, in the extension's crate. The export exists only when the crate is built for
/// WebAssembly; a native build, such as one running the extension's own tests, leaves it out,
/// because the symbols it defines are named after the contract's interfaces and only a
/// WebAssembly link can produce them.
#[macro_export]
macro_rules! export_syntax {
	($syntax:ty) => {
		/// The `parser` export for the extension's syntax, written by `export_syntax!`.
		#[derive(Debug)]
		struct SyntaxExport;

		impl $crate::contract::parser::Guest for SyntaxExport {
			fn describe() -> $crate::contract::syntax::SyntaxDescription {
				<$syntax as $crate::Syntax>::description()
			}

			#[expect(
				clippy::unused_async_trait_impl,
				reason = "the contract requires an asynchronous export; the waiting happens in \
					the task `respond` spawns"
			)]
			async fn parse(
				document: $crate::wit_bindgen::StreamReader<u8>,
				options: $crate::contract::parser::ParseOptions,
			) -> (
				$crate::wit_bindgen::StreamReader<$crate::contract::terms::LocatedStatement>,
				$crate::wit_bindgen::FutureReader<
					::std::result::Result<(), $crate::contract::parser::ParseError>,
				>,
			) {
				$crate::parsers::respond::<$syntax>(document, options)
			}
		}

		#[cfg(target_family = "wasm")]
		$crate::bindings::export!(SyntaxExport with_types_in $crate::bindings);
	};
}
