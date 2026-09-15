# bf-viewer-api

HTTP API serving MARCXML records alongside their BIBFRAME conversion. Reads records by byte offset from a pre-built index. Converts records on demand by using `xsltproc` as subprocess.

## Deployment

This API implements no infrastructure concerns. Rate limiting, access control, TLS, and CORS are deliberately out of scope and must be handled by the layer in front of it. **Do not expose it directly.**

Configuration is read from the environment, `.env` supported via dotenvy. This works nicely on development environment. For production you may want to pass the variables directly.

Startup fails fast if the stylesheets are missing or `xsltproc` is not on PATH. The whole index is loaded into memory, plus a key→position map for O(1) lookup.

### Required

| Variable         | Description                                      |
| ---------------- | ------------------------------------------------ |
| `XML_PATH`       | Source MARCXML collection                        |
| `XML_INDEX_PATH` | Index built by `bf-viewer-cli`                   |
| `XSL_PATH`       | Directory holding the marc2bibframe2 stylesheets |

### Optional

**Note that unparseable env values fall back to the default silently.**

| Variable                     | Default                 | Description                       |
| ---------------------------- | ----------------------- | --------------------------------- |
| `BIND_ADDR`                  | `127.0.0.1:8080`        | Listen address                    |
| `MAX_CONCURRENT_CONVERSIONS` | `2`                     | Max parallel `xsltproc` processes |
| `RUST_LOG`                   | `info,tower_http=debug` | Log filter                        |

## Endpoints

### `GET /info`

Record count and source file size.

### `GET /record`

One record, converted. Accepts either parameter, not both. Without either parameter defaults to using `record_index=0`:

| Parameter      | Description                                                                    |
| -------------- | ------------------------------------------------------------------------------ |
| `record_index` | Zero-based position in the index                                               |
| `key`          | Record key, i.e. combination of records 001+003 in this format: `(<003>)<001>` |

Response carries the raw MARCXML, the BIBFRAME RDF/XML, the RDF flattened to
triples for the frontend, and `has_next` for paging.

## Errors

API errors are returned using Problem JSON format. Internal error details are logged, but never returned.

## Behaviour under load

To prevent conversion from failing under load concurrency is bounded by a semaphore. Excess requests are queued instead of directly failing. Requests however may still fail if a timeout occurs.

## License

MIT
