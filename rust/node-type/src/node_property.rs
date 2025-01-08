// Generated file; do not edit. See `schema-gen` crate.

use common::{serde::{Serialize, Deserialize}, strum::{EnumString, Display}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[strum(serialize_all = "camelCase", crate = "common::strum")]
pub enum NodeProperty {
    About,
    Abstract,
    ActiveSuggestion,
    Address,
    AddressCountry,
    AddressLocality,
    AddressRegion,
    AdmonitionType,
    Affiliations,
    AlternateNames,
    Amounts,
    Annotation,
    Archive,
    Arguments,
    Author,
    Authors,
    AvailableLanguages,
    Bitrate,
    Brands,
    Caption,
    CellType,
    Cells,
    CharacterCount,
    CharacterPercent,
    Chars,
    CitationIntent,
    CitationMode,
    CitationPrefix,
    CitationSuffix,
    Cite,
    ClaimType,
    ClassList,
    Clauses,
    Code,
    CodeLocation,
    CodeRepository,
    CodeSampleType,
    ColumnSpan,
    Columns,
    CommentAspect,
    Comments,
    CompilationDigest,
    CompilationMessages,
    Config,
    ContactPoints,
    Contains,
    Content,
    ContentSize,
    ContentUrl,
    Contributors,
    CostWeight,
    Css,
    DateAccepted,
    DateCreated,
    DateEnd,
    DateModified,
    DatePublished,
    DateReceived,
    DateStart,
    Default,
    Departments,
    DependantNode,
    DependantRelation,
    DependenciesDigest,
    DependenciesFailed,
    DependenciesStale,
    DependencyNode,
    DependencyRelation,
    DeriveAction,
    DeriveFrom,
    DeriveItem,
    DerivedFrom,
    Description,
    Directory,
    Editors,
    Emails,
    EmbedUrl,
    EndColumn,
    EndLine,
    EndPosition,
    ErrorType,
    ExclusiveMaximum,
    ExclusiveMinimum,
    ExecutionBounded,
    ExecutionBounds,
    ExecutionCount,
    ExecutionDependants,
    ExecutionDependencies,
    ExecutionDigest,
    ExecutionDuration,
    ExecutionEnded,
    ExecutionInstance,
    ExecutionMessages,
    ExecutionMode,
    ExecutionPure,
    ExecutionRequired,
    ExecutionStatus,
    ExecutionTags,
    FamilyNames,
    Feedback,
    Files,
    Format,
    FundedBy,
    FundedItems,
    Funders,
    Genre,
    GivenNames,
    Headings,
    Hint,
    HonorificPrefix,
    HonorificSuffix,
    Id,
    Identifiers,
    Images,
    InstructionType,
    InstructionTypes,
    IsActive,
    IsChecked,
    IsCollapsed,
    IsDisabled,
    IsFolded,
    IsGlobal,
    IsInvisible,
    IsPartOf,
    IsSelected,
    IsTemporary,
    Issns,
    IssueNumber,
    Item,
    ItemReviewed,
    ItemType,
    ItemTypes,
    Items,
    ItemsNullable,
    ItemsValidator,
    Iterations,
    JobTitle,
    Keys,
    Keywords,
    Label,
    LabelAutomatically,
    LabelType,
    LastModified,
    LegalName,
    Length,
    Level,
    Licenses,
    Logo,
    Maintainers,
    MathLanguage,
    Mathml,
    MaxItems,
    MaxLength,
    Maximum,
    MediaType,
    MemberOf,
    Members,
    Message,
    Messages,
    MinItems,
    MinLength,
    Minimum,
    MinimumScore,
    ModelIds,
    ModelParameters,
    MultipleOf,
    Name,
    NativeHint,
    NativeType,
    NextBlock,
    NodeCount,
    NodeType,
    NodeTypes,
    NoteType,
    Notes,
    Nulls,
    OperatingSystem,
    Operations,
    Order,
    Otherwise,
    Output,
    Outputs,
    PageEnd,
    PageStart,
    Pagination,
    Parameters,
    ParentItem,
    ParentOrganization,
    Parts,
    Path,
    Pattern,
    Position,
    PostOfficeBoxNumber,
    PostalCode,
    PreviousBlock,
    ProductId,
    ProgrammingLanguage,
    Prompt,
    PropertyId,
    Provenance,
    ProvenanceCategory,
    Publisher,
    QualityWeight,
    Query,
    QueryPatterns,
    RandomSeed,
    References,
    Rel,
    RelativePosition,
    Replacement,
    Replicates,
    Returns,
    ReviewAspect,
    Reviews,
    Role,
    RoleName,
    RowSpan,
    RowType,
    Rows,
    RuntimePlatform,
    SectionType,
    Select,
    SemanticDigest,
    Size,
    SoftwareRequirements,
    SoftwareVersion,
    Source,
    SpeedWeight,
    Sponsors,
    StackTrace,
    StartColumn,
    StartLine,
    StartPosition,
    StateDigest,
    Steps,
    StreetAddress,
    StyleLanguage,
    SuggestionStatus,
    Suggestions,
    Target,
    TargetProducts,
    TelephoneNumbers,
    Temperature,
    Temporary,
    TermCode,
    Text,
    Theme,
    Thumbnail,
    TimeUnit,
    TimeUnits,
    Title,
    Transcript,
    TransferEncoding,
    Type,
    UniqueItems,
    Url,
    Validator,
    Value,
    Values,
    Variable,
    Version,
    VolumeNumber,
}
