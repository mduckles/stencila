# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._inline import Inline
from ._suggestion_inline import SuggestionInline
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class DeleteInline(SuggestionInline):
    """
    A suggestion to delete some inline content.
    """

    type: Literal["DeleteInline"] = field(default="DeleteInline", init=False)

    def __init__(self, content: List[Inline], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        
