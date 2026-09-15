# bf-viewer-cli

A CLI for building index over a large MARCXML file so that individual records can be retrieved without rescanning.

## Usage

    bf-viewer-cli <xml> <index-outfile>

Example:

    bf-viewer-cli data/catalog.xml data/index.tsv

## Index format and key scheme

Please see [bf-viewer-core README](../bf-viewer-core/README.md) for details.

## Important notes

Remember to build CLI using `--release` since it will significantly cut down processing time.

## License

MIT
