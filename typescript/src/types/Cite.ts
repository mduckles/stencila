// Generated file. Do not edit; see `rust/schema-gen` crate.

import { CitationIntent } from './CitationIntent';
import { CitationMode } from './CitationMode';
import { Inline } from './Inline';
import { IntegerOrString } from './IntegerOrString';
import { String } from './String';

// A reference to a CreativeWork that is cited in another CreativeWork.
export class Cite {
  // The type of this item
  type = "Cite";

  // The identifier for this item
  id?: String;

  // The target of the citation (URL or reference ID).
  target: String;

  // Determines how the citation is shown within the surrounding text.
  citationMode: CitationMode;

  // The type/s of the citation, both factually and rhetorically.
  citationIntent?: CitationIntent[];

  // Optional structured content/text of this citation.
  content?: Inline[];

  // The page on which the work starts; for example "135" or "xiii".
  pageStart?: IntegerOrString;

  // The page on which the work ends; for example "138" or "xvi".
  pageEnd?: IntegerOrString;

  // Any description of pages that is not separated into pageStart and pageEnd;
  // for example, "1-6, 9, 55".
  pagination?: String;

  // Text to show before the citation.
  citationPrefix?: String;

  // Text to show after the citation.
  citationSuffix?: String;

  constructor(target: String, citationMode: CitationMode, options?: Cite) {
    if (options) Object.assign(this, options)
    this.target = target;
    this.citationMode = citationMode;
  }
}
