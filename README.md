Group: Rust in Case

# Project Template

This repository is a template for **group and individual contributions** for the Advanced Programming course project, A.Y. 2026/2027.

## Getting Started

This repository **must be forked**.
Do not create a new repository from scratch.
You can rename the fork afterward if desired.

When initializing a library or binary crate, you must add the following crate-level attributes to the crate root:

- For a library crate, add them to `src/lib.rs`.
- For a binary crate, add them to `src/main.rs`.

```
#![deny(unsafe_code)]
#![deny(clippy::pedantic)]
```

These rules are part of the project's coding standards and must be present from the beginning of development.

## README Requirements

You may modify this `README.md` to document your project.
At a minimum, clearly specify:

* The **group name**.
* Which **component(s)** your repository implements.
* Who **contributed to which component(s)**.

Feel free to add any additional information that helps explain the project, its structure, or how to use it.

## AGENTS.md

You can modify `AGENTS.md`, but the current content must be kept as is.

## CLAUDE.md

`CLAUDE.md` must import `@AGENTS.md` at the top.
Do not remove it.

## Rules for code in this repository

These are the project's coding principles, and here they are enforced rather than encouraged.

- **No `unsafe`.**
- **No undocumented panics.** If a function can panic, say so in its doc comment and say when. Most things in here should not panic at all.
- **rustfmt.** Configure your editor.
- Before committing, `cargo fmt --all` must pass. If it does not pass, and the latest commit is not authored by the current user, make a single commit: `style: format code`.


## FAQ

### How do i work on this project?

1. Fork this repository
2. Clone your fork in a directory on your pc
3. Create a new branch
4. Switch to that branch
4. Work on your stuff
5. Commit your changes  
  > [!CAUTION]
  > Do not create one big commit with thousands of changes, break each change into smaller commits
6. Once you're done push your branch
7. Create a Pull Request to merge your changes
8. Once your changes are merged go back to your repo (on github) and sync the merges
9. Return to the repository on your pc
10. Switch to `main` branch
11. Pull your the changes
12. Repeat from step 3

### How do i add an external dependency?

1. Open [Cargo.toml](./Cargo.toml)
2. Go down to `[workspace.dependencies]`
3. Add your dependency (eg. `<DEPENDENCY_NAME> = "<DEPENDENCY_VERSION>"`)
4. Open your package `Cargo.toml` (eg. `chart/Cargo.toml`)
5. Go down to `[dependencies]`
6. Add `<DEPENDENCY_NAME> = { workspace = true }`

### How do i build a package?

```bash
cargo build -p <PACKAGE_NAME>
```

### How do i test a package?

```bash
cargo test -p <PACKAGE_NAME>
```