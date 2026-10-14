# Naamio Space Extension Kit for Rust

A kit for writing Naamio Space extensions in Rust. An extension adds a capability to Naamio
Space by implementing one of its contracts, such as reading a Resource Description
Framework (RDF) syntax. The kit is organised by kind of extension: each kind has a crate
that reduces writing one to the part that is specific to the extension.

## Design

How an extension runs is a detail of its kind, not the kit's organising principle. Today
the extension host runs extensions as WebAssembly components; protocol extensions will
also run as gRPC services, implementing the same contracts. A crate for a kind of
extension hides the mechanics of the way that kind runs, so an extension's own code is the
same whichever way it is delivered.

The shared crate, `naamio-space-kit`, holds what every kind needs. Each kind has
a crate of its own, named after the kind, that builds on it:

- `naamio-space-kit-rdf` is for RDF syntax extensions, which implement the
  `linked-data:rdf` contract. A syntax extension reads one syntax, such as Turtle or
  N-Triples, and reports the statements a document makes, as written; Naamio Space
  resolves, checks and stores what they mean.

Each kind comes with complete example extensions. Their grammars are tested here, natively.
How they behave inside Naamio Space is tested in the
[Naamio Data Space extension host](https://github.com/Naamio/data-space-extensions), which
includes this kit as a Git submodule and runs every example end to end. The host depends
on the kit, and not the other way round, so the repositories form no cycle.

The contracts come from the
[WASI specifications repository](https://github.com/Naamio/data-space-wasi),
included as a Git submodule.

## Writing an RDF syntax extension

An extension is a type that describes its syntax and reads documents into statements,
exported with one macro. The prelude holds everything it usually needs:

```rust
use naamio_space_kit_rdf::prelude::*;

struct NTriples;

impl Syntax for NTriples {
	fn description() -> SyntaxDescription {
		SyntaxDescription {
			name: "N-Triples".to_owned(),
			version: "1.2".to_owned(),
			media_types: vec!["application/n-triples".to_owned()],
			file_extensions: vec!["nt".to_owned()],
			features: SyntaxFeatures::BASE_DIRECTION,
		}
	}

	fn read(document: &[u8], _options: &ParseOptions) -> Reading {
		grammar::read(document)
	}
}

export_syntax!(NTriples);
```

The kit does the rest: the bindings, the asynchronous export, reading the document from
its stream, and writing the statements and the outcome. The statement builders in the
prelude, such as `quad`, `absolute`, `plain_literal` and `located`, build what the grammar
reports. `examples/n-triples` is the complete extension, grammar included.

Build it as a component, then parse a document with the `parse` example in a checkout of
the [extensions repository](https://github.com/Naamio/data-space-extensions):

```shell
cargo build --release --target wasm32-wasip3 -p n-triples-extension
# then, in the extensions repository:
cargo run --example parse -- \
	--component <kit>/target/wasm32-wasip3/release/n_triples_extension.wasm \
	--media-type application/n-triples <document.nt>
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how changes are made and what is checked before
they land.

## Licence

MPL-2.0. Copyright (c) Omnifi Foundation.
