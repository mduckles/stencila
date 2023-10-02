# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Comment = ForwardRef("Comment")
from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class Comment(CreativeWork):
    """
    A comment on an item, e.g on a Article, or SoftwareSourceCode.
    """

    type: Literal["Comment"] = field(default="Comment", init=False)

    parent_item: Optional[Comment] = None
    """The parent comment of this comment."""

    comment_aspect: Optional[str] = None
    """The part or facet of the item that is being commented on."""