# bf-viewer-core

All Rust-based logic. Consumed by CLI and API crates.

## Pipeline

1. CLI: create_index(path) -> Vec<MarcIndexEntry>
2. CLI: write_index(entries, path) -> index.tsv
3. API - main: read_index(path) -> Vec<MarcIndexEntry>
4. API - handler: read_records_as_collection(f, es) -> wrapped MARCXML
5. API - handler: preprocess_collection(xml, dir) -> split MARCXML
6. API - handler: to_bibframe(xml, dir) -> RDF/XML
7. API - handler: rdfxml_to_triples(xml) -> Vec<Triple>
8. API - handler: rdfxml_to_turtle(xml) -> Turtle

## Index file

Key used for indexing created from combination of MARC record fields 001 and 003. The format is `(<003>)<001>`. **Records missing either control field are skipped and not stored to index file.**

Index file fileformat is TSV: `<key>\t<start>\t<length>`.

**Remember to re-generate index file after any change to the source - including whitespace!** Offsets are
not validated on read; a stale index returns wrong bytes, not an error.

## Requirements

- `xsltproc` on PATH
- marc2bibframe2 stylesheets (`third_party/marc2bibframe2`, git submodule)
- Unix, since positional reads use `std::os::unix::fs::FileExt`

## Performance

When build using `--release` indexes approx. 1.3M records/20s on my few years old desktop computer.

## License

MIT
