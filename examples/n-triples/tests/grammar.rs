// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! Feature tests for the N-Triples example's grammar, run natively.
//!
//! The grammar is the only part of the example that is specific to N-Triples, so it is the
//! part worth testing on its own: these tests call it directly, without building a component
//! or starting a host, which keeps them fast enough to run on every change. They cover each
//! part of the N-Triples grammar: absolute Internationalized Resource Identifiers (IRIs),
//! blank node labels, the literal forms and their escapes, comments and blank lines, and the
//! errors a person fixing a document would need to find. How the example behaves inside a
//! host is tested from the host's side, in the extensions repository.

use std::error::Error;

use n_triples_extension::grammar::read;
use naamio_space_kit_rdf::prelude::*;

/// A boxed error type broad enough for anything a test below might fail on.
type TestResult = Result<(), Box<dyn Error>>;

/// Reads `document` and returns its statements without their spans, for tests about what a
/// document says. A document the grammar refuses fails the test with the grammar's error.
fn statements(document: &str) -> Result<Vec<Statement>, Box<dyn Error>> {
	let reading = read(document.as_bytes());
	reading.outcome.map_err(|error| format!("{error:?}"))?;
	Ok(reading
		.statements
		.into_iter()
		.map(|located| located.statement)
		.collect())
}

/// Reads a one-triple `document` and returns the triple's object, for tests about how a
/// single term is read. Any other number of statements fails the test.
fn only_object(document: &str) -> Result<Term, Box<dyn Error>> {
	match statements(document)?.as_slice() {
		[Statement::Quad(quad)] => Ok(quad.object.clone()),
		other => Err(format!("expected one quad, but the grammar reported {other:?}").into()),
	}
}

/// Reads `document` and returns the error that stopped the reading, for tests about
/// refusals. A document the grammar accepts fails the test.
fn error(document: &str) -> Result<ParseError, Box<dyn Error>> {
	match read(document.as_bytes()).outcome {
		Err(error) => Ok(error),
		Ok(()) => Err(format!("expected an error for {document:?}").into()),
	}
}

/// N-Triples writes every IRI in full, so the grammar reports each one as absolute and the
/// host uses it exactly as written. Reporting one as relative would ask the host to resolve
/// it, and resolution removes dot segments, changing IRIs the document had settled.
#[test]
fn reads_a_triple_of_absolute_iris() -> TestResult {
	let found =
		statements("<https://example.org/s> <https://example.org/p> <https://example.org/o> .\n")?;

	assert_eq!(
		found,
		vec![quad(
			absolute("https://example.org/s"),
			absolute("https://example.org/p"),
			absolute("https://example.org/o"),
		)]
	);

	Ok(())
}

/// A label may contain dots, as in `_:a.b`, but in `_:c.` the dot is the triple's full stop.
/// Taking it into the label would leave the triple without its full stop, and make `_:c.`
/// and `_:c` two different nodes.
#[test]
fn reads_blank_node_labels_without_a_trailing_dot() -> TestResult {
	let found = statements("_:a.b <https://example.org/p> _:c.\n")?;

	assert_eq!(
		found,
		vec![quad(
			labelled_blank_node("a.b"),
			absolute("https://example.org/p"),
			labelled_blank_node("c")
		)]
	);

	Ok(())
}

/// The contract carries terms with their escapes decoded, so the host stores the characters
/// the author meant rather than the escape sequences that wrote them. This covers character
/// escapes such as `\t` and `\"`, and numeric escapes in both their four-digit and eight-digit
/// forms.
#[test]
fn decodes_the_escapes_in_a_literal() -> TestResult {
	let object = only_object(
		"<https://example.org/s> <https://example.org/p> \"tab\\there \\\"quoted\\\" \\u00e9\\U0001F600\" .",
	)?;

	assert_eq!(object, plain_literal("tab\there \"quoted\" é😀").into());

	Ok(())
}

/// The grammar only separates a language tag from the RDF 1.2 base direction written after
/// two hyphens. Turning them into an `rdf:langString` or `rdf:dirLangString`, and checking the
/// tag, is the host's work.
#[test]
fn reads_language_tags_and_base_directions() -> TestResult {
	let tagged = only_object(r#"<https://example.org/s> <https://example.org/p> "Hei"@fi ."#)?;
	let directed =
		only_object(r#"<https://example.org/s> <https://example.org/p> "مرحبا"@ar--rtl ."#)?;

	assert_eq!(tagged, language_literal("Hei", "fi", None).into());
	assert_eq!(
		directed,
		language_literal("مرحبا", "ar", Some(BaseDirection::Rtl)).into()
	);

	Ok(())
}

/// A datatype after `^^` is an IRI like any other in N-Triples: written in full, and reported
/// as absolute for the host to check.
#[test]
fn reads_typed_literals() -> TestResult {
	let object = only_object(
		r#"<https://example.org/s> <https://example.org/p> "42"^^<http://www.w3.org/2001/XMLSchema#integer> ."#,
	)?;

	assert_eq!(
		object,
		typed_literal("42", absolute("http://www.w3.org/2001/XMLSchema#integer")).into()
	);

	Ok(())
}

/// Blank lines, comment lines and comments after a triple produce no statements. A `#` inside
/// an IRI or a literal is part of the term, because fragment identifiers such as `p#q` are
/// common in vocabularies, and treating them as comments would cut the IRI short.
#[test]
fn ignores_comments_and_blank_lines() -> TestResult {
	let document = "# a comment\n\n<https://example.org/s> <https://example.org/p#q> \"#kept\" . # trailing\n\t\n";

	let found = statements(document)?;

	assert_eq!(
		found,
		vec![quad(
			absolute("https://example.org/s"),
			absolute("https://example.org/p#q"),
			plain_literal("#kept")
		)]
	);

	Ok(())
}

/// Each quad carries the span of its object, the term that completed it, as the contract
/// specifies, so that a problem the host finds in the quad points somewhere a person can
/// look. Here the literal `"o"` occupies bytes 48 to 51.
#[test]
fn places_each_quad_at_its_object() {
	let document = "<https://example.org/s> <https://example.org/p> \"o\" .\n";

	let reading = read(document.as_bytes());

	let spans: Vec<Span> = reading
		.statements
		.iter()
		.map(|located| located.span)
		.collect();
	assert_eq!(spans, vec![span(48, 51)]);
}

/// Each kind of broken line is refused as a syntax error whose span starts where the problem
/// is: at the relative IRI, at the end of a line missing its full stop, at the text after a
/// full stop, at a literal in the subject position, and at the quotation mark of a string
/// that is never closed. The host turns that byte into the line and column a person needs.
#[test]
fn refuses_lines_that_break_the_grammar() -> TestResult {
	let cases = [
		(
			"a relative IRI",
			"<alice> <https://example.org/p> \"o\" .",
			0,
		),
		(
			"a missing full stop",
			"<https://example.org/s> <https://example.org/p> \"o\"",
			51,
		),
		(
			"text after the full stop",
			"<https://example.org/s> <https://example.org/p> \"o\" . extra",
			54,
		),
		(
			"a literal as a subject",
			"\"Alice\" <https://example.org/p> \"o\" .",
			0,
		),
		(
			"a string never closed",
			"<https://example.org/s> <https://example.org/p> \"open .",
			48,
		),
	];

	for (case, line, expected_start) in cases {
		let ParseError::Syntax(error) = error(line)? else {
			return Err(format!("{case} should be a syntax error").into());
		};
		assert_eq!(
			error.span.start, expected_start,
			"{case}: {}",
			error.message
		);
	}

	Ok(())
}

/// The contract has a parser deliver every statement it completed before an error. The
/// triple on the first line arrives with the error about the second, and the host decides
/// whether to keep it.
#[test]
fn keeps_the_triples_before_an_error() {
	let document = "<https://example.org/s> <https://example.org/p> \"one\" .\n<alice> <https://example.org/p> \"two\" .\n";

	let reading = read(document.as_bytes());

	assert_eq!(reading.statements.len(), 1);
	assert!(reading.outcome.is_err());
}

/// RDF 1.2 triple terms are valid N-Triples 1.2 that this example does not read. Reporting
/// them as unsupported, rather than as a syntax error, tells a person the document may well be
/// correct and that a fuller parser would read it.
#[test]
fn reports_triple_terms_as_unsupported() -> TestResult {
	let document = "<https://example.org/s> <https://example.org/p> <<( <https://example.org/a> <https://example.org/b> <https://example.org/c> )>> .";

	assert!(matches!(error(document)?, ParseError::Unsupported(_)));

	Ok(())
}

/// N-Triples documents must be UTF-8. A document that is not is refused with a span at the
/// first byte that breaks the encoding, here byte 21, so the person can find the corrupted
/// text rather than being told only that the file is wrong.
#[test]
fn refuses_a_document_that_is_not_utf8() -> TestResult {
	let ParseError::Syntax(error) = read(b"<https://example.org/\xff> .")
		.outcome
		.err()
		.ok_or("expected an error")?
	else {
		return Err("expected a syntax error".into());
	};

	assert_eq!(error.span.start, 21);

	Ok(())
}
