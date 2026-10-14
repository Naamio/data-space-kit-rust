# Contributing to the Naamio Space Extension Kit for Rust

Start with the [README](README.md) for what this kit is and who uses it.

## Getting the source

The contracts live in their own repository and are included here as a Git submodule. Clone
with it:

```shell
git clone --recurse-submodules git@github.com:Naamio/data-space-kit-rust.git
```

Do not edit the contracts inside the submodule. Change them in the
[WASI specifications repository](https://github.com/Naamio/data-space-wasi),
then move the submodule forward here in its own merge request.

The kit must not include the extension host, or anything that includes it, as a submodule
or a dependency. The host includes the kit, and a dependency in the other direction would
form a cycle that recursive clones and Cargo would both follow.

## Tests first

Write the failing test, then the smallest code that passes it, then improve what is there if
improving it is worth doing. Work one feature at a time: each feature's tests fail first,
then pass, before the next feature starts.

A test documents behaviour and treats the implementation as a black box. Name it for what it
establishes, and say in its documentation why that matters, so someone changing the code
later knows what they would be breaking.

Each kind's examples are complete extensions. Their grammars are tested here, natively.
Their behaviour inside Naamio Space is tested end to end in the
[extensions repository](https://github.com/Naamio/data-space-extensions), which includes
this kit as a submodule: a change to an example's behaviour comes with a change to those
tests.

## Gates

```shell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-wasip3 -- -D warnings
cargo test --workspace
```

Clippy runs at `deny` for `all`, `pedantic`, and the lints named individually in
`[workspace.lints]`, with thresholds in `.clippy.toml`. Do not weaken a lint to make code
compile: change the code, or if the lint is genuinely wrong for a case, mark that one case
with `#[expect(...)]` and a `reason` explaining it. Use `expect` and not `allow`, so the
exemption is reported once it stops being needed.

`unsafe_code` is forbidden throughout, and an extension built with the kit keeps that
setting. The code a bindings generator writes is exempt only where it is expanded from
another crate's macro, so each kind's crate generates its contract's bindings and exports
extensions through a macro of its own. An extension that called the generator directly
would expand `unsafe` code in its own crate, which `forbid` would refuse.

Every `.rs` file and every `Cargo.toml` carries an SPDX header, in the comment syntax for
that file type:

```rust
// SPDX-FileCopyrightText: 2026 Omnifi Foundation
// SPDX-License-Identifier: MPL-2.0
```

## Comments

Public items are documented with their arguments, what they return, what they panic on, and
an example where one helps. Private items are documented too; `missing_docs_in_private_items`
is denied.

Write for someone reading English as an additional language, and for someone meeting the code
for the first time. Say what something does and why it is that way. Do not explain the build
configuration in the file, and do not write about the writing.

## Names

No abbreviations: `configuration`, not `config`. `misc`, `common`, `core`, `utils`, `shared`
and `types` are refused as crate or module names, each one a place a decision got deferred
rather than made. Prefer plural module names where they make sense. "User" is refused too:
name the person by their role, such as *account holder*, *operator* or *developer*.

## Commits and merge requests

Commit messages are a single line, in the imperative, specific enough to identify the
change. Detail belongs in the merge request.

Keep unrelated changes apart. A rename, a formatting pass, and a behaviour change are three
merge requests, because reviewing them together means reviewing none of them properly.

## Versions

Check a version before writing it into a manifest. The toolchain is a nightly release
pinned by date in `rust-toolchain.toml`, the same release the extension host uses, because
components target `wasm32-wasip3` from a nightly release. Bump it deliberately, in its own
merge request.
