# Directory

**A directory on the file system.**

Previously this type extended `Collection` (which in turn extends `CreativeWork`).
However, to avoid consuming more memory that necessary when creating directory listings
with many directories, it now extends `Entity`.


**`@id`**: `stencila:Directory`

## Properties

The `Directory` type has these properties:

| Name    | Aliases            | `@id`                                            | Type                                                                                                                                                                                                      | Description                                                     | Inherited from                                                                                   |
| ------- | ------------------ | ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`    | -                  | [`schema:id`](https://schema.org/id)             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                           | The identifier for this item.                                   | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`  | -                  | [`schema:name`](https://schema.org/name)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                           | The name of the directory.                                      | -                                                                                                |
| `path`  | -                  | `stencila:path`                                  | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                           | The path (absolute or relative) of the file on the file system. | -                                                                                                |
| `parts` | `hasParts`, `part` | [`schema:hasParts`](https://schema.org/hasParts) | ([`File`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md) \| [`Directory`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/directory.md))* | The files and other directories within this directory.          | -                                                                                                |

## Related

The `Directory` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Directory` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |       |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              | 🟢 No loss  | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `Directory` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Directory.jsonld)
- [JSON Schema](https://stencila.org/Directory.schema.json)
- Python class [`Directory`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/directory.py)
- Rust struct [`Directory`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/directory.rs)
- TypeScript class [`Directory`](https://github.com/stencila/stencila/blob/main/ts/src/types/Directory.ts)

## Source

This documentation was generated from [`Directory.yaml`](https://github.com/stencila/stencila/blob/main/schema/Directory.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
