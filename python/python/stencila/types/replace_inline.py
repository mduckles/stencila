# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .suggestion_inline import SuggestionInline


@dataclass(init=False)
class ReplaceInline(SuggestionInline):
    """
    A suggestion to replace some inline content with new inline content.
    """

    type: Literal["ReplaceInline"] = field(default="ReplaceInline", init=False)

    replacement: List[Inline]
    """The new replacement inline content."""

    def __init__(self, content: List[Inline], replacement: List[Inline], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        self.replacement = replacement
