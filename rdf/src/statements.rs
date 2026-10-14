// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0

//! Builders for the contract's terms and statements.
//!
//! The contract's types nest deeply: a quad holds a node that holds an Internationalized
//! Resource Identifier (IRI) reference that holds a string. These functions build each
//! value from what an extension's grammar has in hand, and the conversions below let a
//! builder take an IRI reference or a blank node wherever the contract accepts either, so a
//! grammar writes `quad(subject, predicate, object)` without wrapping each term in the
//! variant its position needs.
//!
//! Nothing here is checked. Whether an IRI is valid or a prefix declared is the host's
//! decision, made when it interprets the statement.

use crate::contract::parser::{ParseError, SyntaxError};
use crate::contract::terms::{
	Annotation, BaseDirection, BlankNode, IriReference, Language, Literal, LocatedStatement, Node,
	PrefixedName, Quad, Span, Statement, Term,
};

/// An IRI written with a scheme, which the host uses exactly as written.
#[must_use]
pub fn absolute(iri: &str) -> IriReference {
	IriReference::Absolute(iri.to_owned())
}

/// An IRI written without a scheme, which the host resolves against the base in effect.
#[must_use]
pub fn relative(reference: &str) -> IriReference {
	IriReference::Relative(reference.to_owned())
}

/// A prefixed name such as `ex:alice`, which the host expands against the prefixes the
/// document declared.
#[must_use]
pub fn prefixed(prefix: &str, local: &str) -> IriReference {
	IriReference::Prefixed(PrefixedName {
		prefix: prefix.to_owned(),
		local: local.to_owned(),
	})
}

/// A blank node the document labels, such as `_:bob`, given without the `_:`.
#[must_use]
pub fn labelled_blank_node(label: &str) -> BlankNode {
	BlankNode::Written(label.to_owned())
}

/// A blank node the document does not label, such as Turtle's `[ ]`, numbered by the
/// extension. Each number must stand for one node only within a document.
#[must_use]
pub const fn numbered_blank_node(number: u64) -> BlankNode {
	BlankNode::Generated(number)
}

/// A literal with neither a datatype nor a language, which the host stores as an
/// `xsd:string`.
#[must_use]
pub fn plain_literal(lexical_form: &str) -> Literal {
	Literal {
		lexical_form: lexical_form.to_owned(),
		annotation: None,
	}
}

/// A literal with a datatype, such as `"42"^^xsd:integer`.
#[must_use]
pub fn typed_literal(lexical_form: &str, datatype: IriReference) -> Literal {
	Literal {
		lexical_form: lexical_form.to_owned(),
		annotation: Some(Annotation::Datatype(datatype)),
	}
}

/// A literal with a language tag, and optionally a base direction, such as `"Hei"@fi`.
#[must_use]
pub fn language_literal(
	lexical_form: &str,
	tag: &str,
	direction: Option<BaseDirection>,
) -> Literal {
	Literal {
		lexical_form: lexical_form.to_owned(),
		annotation: Some(Annotation::Language(Language {
			tag: tag.to_owned(),
			direction,
		})),
	}
}

/// A quad in the default graph that uses no triple terms: the statement most syntaxes make
/// for an ordinary triple.
pub fn quad(
	subject: impl Into<Node>,
	predicate: IriReference,
	object: impl Into<Term>,
) -> Statement {
	Statement::Quad(Quad {
		subject: subject.into(),
		predicate,
		object: object.into(),
		graph_name: None,
		triple_terms: Vec::new(),
	})
}

/// The bytes from `start` to `end` of a document, as a grammar counts them while it reads.
#[must_use]
pub fn span(start: usize, end: usize) -> Span {
	Span {
		start: u64::try_from(start).unwrap_or(u64::MAX),
		end: u64::try_from(end).unwrap_or(u64::MAX),
	}
}

/// `statement`, placed at `span` of the document. For a quad, the span should cover the term
/// that completed it, so that a host pointing at a problem points at the right object.
#[must_use]
pub const fn located(span: Span, statement: Statement) -> LocatedStatement {
	LocatedStatement { span, statement }
}

/// The error for a part of the document, at `span`, that is not valid in the syntax.
#[must_use]
pub fn syntax_error(message: &str, span: Span) -> ParseError {
	ParseError::Syntax(SyntaxError {
		message: message.to_owned(),
		span,
	})
}

impl From<IriReference> for Node {
	fn from(reference: IriReference) -> Self {
		Self::Iri(reference)
	}
}

impl From<BlankNode> for Node {
	fn from(blank_node: BlankNode) -> Self {
		Self::BlankNode(blank_node)
	}
}

impl From<IriReference> for Term {
	fn from(reference: IriReference) -> Self {
		Self::Iri(reference)
	}
}

impl From<BlankNode> for Term {
	fn from(blank_node: BlankNode) -> Self {
		Self::BlankNode(blank_node)
	}
}

impl From<Literal> for Term {
	fn from(literal: Literal) -> Self {
		Self::Literal(literal)
	}
}

impl From<Node> for Term {
	fn from(node: Node) -> Self {
		match node {
			Node::Iri(reference) => Self::Iri(reference),
			Node::BlankNode(blank_node) => Self::BlankNode(blank_node),
		}
	}
}
