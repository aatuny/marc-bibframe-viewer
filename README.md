# marc-bibframe-viewer

Web tool for viewing a MARC record alongside its BIBFRAME conversion.

This was a personal project to learn more about Rust and BIBFRAME and also to investigate AI assisted learning strategies.

## Approach

By creating a byte-offset index beforehand using a CLI, API may find a record from large MARCXML files in O(1) with reasonable memory requirements. This combined with spawning a subprocess for `xsltproc` allows API handler to provide a performant per record conversion without needing a separate triple store.

The conversion rules are from LoC's marc2bibframe2 which is included as a third-party submodule to this repository.

## Layout

    Cargo.toml                    # virtual workspace
    crates/bf-viewer-core/        # lib - logic
    crates/bf-viewer-cli/         # bin - index building
    crates/bf-viewer-api/         # bin - http server
    frontend/                     # lightweight Astro components
    third_party/                  # third-party submodules

Each crate and the frontend has its own README.

## Setting development environment

Requires `xsltproc` on PATH. Unix only as positional reads utilize `std::os::unix::fs::FileExt`.

1. Initialize third-party git submodules by using: `git submodule update --init --recursive`
2. Build release version of CLI: `cargo build --release -p bf-viewer-cli`
3. Build the index for source XML: `./target/release/bf-viewer-cli data/catalog.xml data/index.tsv`
4. Setup .env file for API. See example at `.env.example` (or just rename to .env).
5. (Optional) if you started API development server elsewhere than port 8080, fix `frontend/astro.config.mjs` server proxy appropriately
6. Start API in shell #1: `cargo run -p bf-viewer-api`
7. Start UI in shell #2: `cd frontend && npm i && npm run dev`
8. Browse to `localhost:4321`

## Use of AI disclosure

This learning project was developed with the help of Claude (chat mode available at claude.ai). The code creation loop was as follows:

1. Author prompted Claude on what feature was wanted to be achieved
2. Claude proposed a solution, which the author reviewed and refined
3. Claude wrote initial code, which the author then reviewed and edited

## License

The source code found in this repository is licensed under MIT license.

Please note that a binary release would most probably require additional steps and that submodules and spawned subprocesses carry their own respective licenses.
