// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! The N-Triples grammar: one triple per line, every Internationalized Resource Identifier
//! (IRI) absolute, no prefixes and no shorthands.
//!
//! ```text
//! triple   ::= subject predicate object '.'
//! subject  ::= IRIREF | BLANK_NODE_LABEL
//! object   ::= IRIREF | BLANK_NODE_LABEL | literal
//! literal  ::= STRING_LITERAL_QUOTE ('^^' IRIREF | LANG_DIR)?
//! ```
//!
//! Because N-Triples has no relative IRIs, every IRI is reported as absolute, and the host
//! uses it exactly as written. The grammar checks that each one has a scheme, since a
//! relative IRI is a syntax error in N-Triples, not something for the host to resolve.

use naamio_space_kit_rdf::prelude::*;

/// Reads an N-Triples document, keeping every triple before the first error.
#[must_use]
pub fn read(document: &[u8]) -> Reading {
	let text = match std::str::from_utf8(document) {
		Ok(text) => text,
		Err(invalid) => {
			let at = invalid.valid_up_to();
			return Reading {
				statements: Vec::new(),
				outcome: Err(syntax_error(
					"the document is not valid UTF-8",
					span(at, at + 1),
				)),
			};
		}
	};
	let mut reader = Reader { text, position: 0 };
	let mut statements = Vec::new();
	let outcome = loop {
		match reader.line() {
			Ok(Line::Triple(statement)) => statements.push(*statement),
			Ok(Line::Empty) => {}
			Ok(Line::EndOfDocument) => break Ok(()),
			Err(error) => break Err(error),
		}
	};
	Reading {
		statements,
		outcome,
	}
}

/// The contents of one line of an N-Triples document.
enum Line {
	/// The document has no more lines.
	EndOfDocument,
	/// A line with no triple: blank, or only a comment.
	Empty,
	/// A triple, as a quad statement placed at the span of its object. Boxed, because a
	/// statement is far larger than the other variants.
	Triple(Box<LocatedStatement>),
}

/// A position in an N-Triples document.
struct Reader<'text> {
	/// The document.
	text: &'text str,
	/// The byte offset of the next character to read.
	position: usize,
}

impl Reader<'_> {
	/// Reads the next line.
	fn line(&mut self) -> Result<Line, ParseError> {
		if self.position >= self.text.len() {
			return Ok(Line::EndOfDocument);
		}
		self.skip_blanks();
		if self.at_end_of_line() {
			self.next_line();
			return Ok(Line::Empty);
		}
		let subject = self.subject()?;
		self.skip_blanks();
		let predicate = self.iri()?;
		self.skip_blanks();
		let (object, object_span) = self.object()?;
		self.skip_blanks();
		self.expect('.', "a triple must end with `.`")?;
		self.skip_blanks();
		if !self.at_end_of_line() {
			return Err(self.error_here("nothing may follow a triple on its line but a comment"));
		}
		self.next_line();
		Ok(Line::Triple(Box::new(located(
			object_span,
			quad(subject, predicate, object),
		))))
	}

	/// `subject ::= IRIREF | BLANK_NODE_LABEL`
	fn subject(&mut self) -> Result<Node, ParseError> {
		if self.rest().starts_with("_:") {
			return Ok(self.blank_node_label().into());
		}
		Ok(self.iri()?.into())
	}

	/// `object ::= IRIREF | BLANK_NODE_LABEL | literal`, with the span it came from.
	fn object(&mut self) -> Result<(Term, Span), ParseError> {
		let start = self.position;
		let term: Term = if self.rest().starts_with("<<(") {
			return Err(ParseError::Unsupported(
				"triple terms, written `<<( … )>>`".to_owned(),
			));
		} else if self.rest().starts_with('<') {
			self.iri()?.into()
		} else if self.rest().starts_with("_:") {
			self.blank_node_label().into()
		} else if self.rest().starts_with('"') {
			self.literal()?.into()
		} else {
			return Err(self.error_here("an object must be an IRI, a blank node or a literal"));
		};
		Ok((term, span(start, self.position)))
	}

	/// `IRIREF`, which N-Triples requires to be absolute.
	fn iri(&mut self) -> Result<IriReference, ParseError> {
		let start = self.position;
		self.expect('<', "an IRI must be written in angle brackets")?;
		let mut iri = String::new();
		loop {
			match self.next_char() {
				Some('>') => break,
				Some('\\') => iri.push(self.numeric_escape(start)?),
				Some(character) if character <= ' ' || "<\"{}|^`".contains(character) => {
					return Err(syntax_error(
						&format!("{character:?} is not allowed in an IRI"),
						span(start, self.position),
					));
				}
				Some(character) => iri.push(character),
				None => {
					return Err(syntax_error(
						"the IRI is never closed with `>`",
						span(start, self.position),
					));
				}
			}
		}
		if !has_scheme(&iri) {
			return Err(syntax_error(
				"N-Triples requires every IRI to be absolute",
				span(start, self.position),
			));
		}
		Ok(absolute(&iri))
	}

	/// `BLANK_NODE_LABEL`, the `_:` already seen.
	fn blank_node_label(&mut self) -> BlankNode {
		self.position += 2;
		let start = self.position;
		while self
			.peek()
			.is_some_and(|character| character.is_alphanumeric() || "_-.".contains(character))
		{
			self.position += self.peek().map_or(1, char::len_utf8);
		}
		while self.text[start..self.position].ends_with('.') {
			self.position -= 1;
		}
		labelled_blank_node(&self.text[start..self.position])
	}

	/// `literal ::= STRING_LITERAL_QUOTE ('^^' IRIREF | LANG_DIR)?`
	fn literal(&mut self) -> Result<Literal, ParseError> {
		let start = self.position;
		self.position += 1;
		let mut value = String::new();
		loop {
			match self.next_char() {
				Some('"') => break,
				Some('\\') => value.push(self.string_escape(start)?),
				Some('\n' | '\r') | None => {
					return Err(syntax_error(
						"the string is never closed on its line",
						span(start, start + 1),
					));
				}
				Some(character) => value.push(character),
			}
		}
		if self.rest().starts_with("^^") {
			self.position += 2;
			return Ok(typed_literal(&value, self.iri()?));
		}
		if self.rest().starts_with('@') {
			let (tag, direction) = self.language();
			return Ok(language_literal(&value, &tag, direction));
		}
		Ok(plain_literal(&value))
	}

	/// `LANG_DIR`, the `@` already seen: a language tag and an optional `--ltr` or `--rtl`.
	fn language(&mut self) -> (String, Option<BaseDirection>) {
		self.position += 1;
		let start = self.position;
		while self
			.peek()
			.is_some_and(|character| character.is_ascii_alphanumeric() || character == '-')
			&& !self.rest().starts_with("--")
		{
			self.position += 1;
		}
		let tag = self.text[start..self.position].to_owned();
		let direction = if self.rest().starts_with("--ltr") {
			Some(BaseDirection::Ltr)
		} else if self.rest().starts_with("--rtl") {
			Some(BaseDirection::Rtl)
		} else {
			None
		};
		if direction.is_some() {
			self.position += "--ltr".len();
		}
		(tag, direction)
	}

	/// The character a string escape such as `\n` or `é` stands for, the backslash
	/// already read.
	fn string_escape(&mut self, start: usize) -> Result<char, ParseError> {
		match self.next_char() {
			Some('t') => Ok('\t'),
			Some('b') => Ok('\u{8}'),
			Some('n') => Ok('\n'),
			Some('r') => Ok('\r'),
			Some('f') => Ok('\u{c}'),
			Some('"') => Ok('"'),
			Some('\'') => Ok('\''),
			Some('\\') => Ok('\\'),
			Some(marker @ ('u' | 'U')) => self.hexadecimal(marker, start),
			_ => Err(syntax_error(
				"this escape is not allowed in a string",
				span(start, self.position),
			)),
		}
	}

	/// The character a numeric escape in an IRI stands for, the backslash already read.
	fn numeric_escape(&mut self, start: usize) -> Result<char, ParseError> {
		match self.next_char() {
			Some(marker @ ('u' | 'U')) => self.hexadecimal(marker, start),
			_ => Err(syntax_error(
				"only `\\u` and `\\U` escapes are allowed in an IRI",
				span(start, self.position),
			)),
		}
	}

	/// The character named by the four (`u`) or eight (`U`) hexadecimal digits that follow.
	fn hexadecimal(&mut self, marker: char, start: usize) -> Result<char, ParseError> {
		let length = if marker == 'u' { 4 } else { 8 };
		let digits = self
			.text
			.get(self.position..self.position + length)
			.unwrap_or_default();
		self.position += digits.len();
		u32::from_str_radix(digits, 16)
			.ok()
			.filter(|_| digits.len() == length)
			.and_then(char::from_u32)
			.ok_or_else(|| {
				syntax_error(
					"the escape does not name a Unicode character",
					span(start, self.position),
				)
			})
	}

	/// Takes `expected`, or reports `problem` at the current position.
	fn expect(&mut self, expected: char, problem: &str) -> Result<(), ParseError> {
		if self.peek() == Some(expected) {
			self.position += expected.len_utf8();
			return Ok(());
		}
		Err(self.error_here(problem))
	}

	/// A syntax error at the current position, one character wide.
	fn error_here(&self, problem: &str) -> ParseError {
		let width = self.peek().map_or(0, char::len_utf8);
		syntax_error(problem, span(self.position, self.position + width))
	}

	/// Whether the rest of the line is empty or a comment.
	fn at_end_of_line(&self) -> bool {
		matches!(self.peek(), None | Some('\n' | '\r' | '#'))
	}

	/// Moves to the start of the next line.
	fn next_line(&mut self) {
		self.position = self.text[self.position..]
			.find('\n')
			.map_or(self.text.len(), |offset| self.position + offset + 1);
	}

	/// Moves past spaces and tabs.
	fn skip_blanks(&mut self) {
		while matches!(self.peek(), Some(' ' | '\t')) {
			self.position += 1;
		}
	}

	/// Takes the next character.
	fn next_char(&mut self) -> Option<char> {
		let character = self.peek()?;
		self.position += character.len_utf8();
		Some(character)
	}

	/// The next character, without taking it.
	fn peek(&self) -> Option<char> {
		self.rest().chars().next()
	}

	/// The document from the current position on.
	fn rest(&self) -> &str {
		&self.text[self.position..]
	}
}

/// Whether an IRI starts with a scheme, which is what makes it absolute.
fn has_scheme(iri: &str) -> bool {
	let Some((scheme, _)) = iri.split_once(':') else {
		return false;
	};
	let mut characters = scheme.chars();
	characters
		.next()
		.is_some_and(|first| first.is_ascii_alphabetic())
		&& characters
			.all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
}
