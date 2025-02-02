# Section

**A section of a document.**

**`@id`**: `stencila:Section`

## Properties

The `Section` type has these properties:

| Name          | Aliases                        | `@id`                                        | Type                                                                                                                 | Description                                                    | Inherited from                                                                                   |
| ------------- | ------------------------------ | -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`          | -                              | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content`     | -                              | `stencila:content`                           | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                      | The content within the section.                                | -                                                                                                |
| `sectionType` | `section-type`, `section_type` | `stencila:sectionType`                       | [`SectionType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section-type.md)          | The type of section.                                           | -                                                                                                |
| `authors`     | `author`                       | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                    | The authors of the section.                                    | -                                                                                                |
| `provenance`  | -                              | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)* | A summary of the provenance of the content within the section. | -                                                                                                |

## Related

The `Section` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Section` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                                                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ------------------------------------------------------------------------------------------------------------------ |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                                                                                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🟢 No loss    |            | 🚧 Under development | Encoded as [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section) using special function |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🟢 No loss  | 🚧 Under development | Encoded as [`<sec>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sec.html)                   |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              | Encoded using implemented function                                                                                 |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                                                                                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                    |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                    |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                    |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                                                                                                    |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                                                                                                    |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                                                                                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                                                    |

## Bindings

The `Section` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Section.jsonld)
- [JSON Schema](https://stencila.org/Section.schema.json)
- Python class [`Section`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/section.py)
- Rust struct [`Section`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section.rs)
- TypeScript class [`Section`](https://github.com/stencila/stencila/blob/main/ts/src/types/Section.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Section` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property      | Complexity | Description                                                 | Strategy                               |
| ------------- | ---------- | ----------------------------------------------------------- | -------------------------------------- |
| `content`     | Min+       | An empty vector                                             | `Vec::new()`                           |
|               | Low+       | Generate an arbitrary heading and an arbitrary paragraph.   | `vec_heading_paragraph()`              |
|               | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)`          |
|               | Max        | Generate up to eight arbitrary, non-recursive, block nodes. | `vec_blocks_non_recursive(8)`          |
| `sectionType` | Min+       | No type.                                                    | `None`                                 |
|               | Low+       | Generate an arbitrary section type.                         | `option::of(SectionType::arbitrary())` |

## Source

This documentation was generated from [`Section.yaml`](https://github.com/stencila/stencila/blob/main/schema/Section.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
