# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class CitationIntent(StrEnum):
    """
    The type or nature of a citation, both factually and rhetorically.
    """

    AgreesWith = "AgreesWith"
    CitesAsAuthority = "CitesAsAuthority"
    CitesAsDataSource = "CitesAsDataSource"
    CitesAsEvidence = "CitesAsEvidence"
    CitesAsMetadataDocument = "CitesAsMetadataDocument"
    CitesAsPotentialSolution = "CitesAsPotentialSolution"
    CitesAsRecommendedReading = "CitesAsRecommendedReading"
    CitesAsRelated = "CitesAsRelated"
    CitesAsSourceDocument = "CitesAsSourceDocument"
    CitesForInformation = "CitesForInformation"
    Compiles = "Compiles"
    Confirms = "Confirms"
    ContainsAssertionFrom = "ContainsAssertionFrom"
    Corrects = "Corrects"
    Credits = "Credits"
    Critiques = "Critiques"
    Derides = "Derides"
    Describes = "Describes"
    DisagreesWith = "DisagreesWith"
    Discusses = "Discusses"
    Disputes = "Disputes"
    Documents = "Documents"
    Extends = "Extends"
    GivesBackgroundTo = "GivesBackgroundTo"
    GivesSupportTo = "GivesSupportTo"
    HasReplyFrom = "HasReplyFrom"
    IncludesExcerptFrom = "IncludesExcerptFrom"
    IncludesQuotationFrom = "IncludesQuotationFrom"
    IsAgreedWithBy = "IsAgreedWithBy"
    IsCitedAsAuthorityBy = "IsCitedAsAuthorityBy"
    IsCitedAsDataSourceBy = "IsCitedAsDataSourceBy"
    IsCitedAsEvidenceBy = "IsCitedAsEvidenceBy"
    IsCitedAsMetadataDocumentBy = "IsCitedAsMetadataDocumentBy"
    IsCitedAsPotentialSolutionBy = "IsCitedAsPotentialSolutionBy"
    IsCitedAsRecommendedReadingBy = "IsCitedAsRecommendedReadingBy"
    IsCitedAsRelatedBy = "IsCitedAsRelatedBy"
    IsCitedAsSourceDocumentBy = "IsCitedAsSourceDocumentBy"
    IsCitedBy = "IsCitedBy"
    IsCitedForInformationBy = "IsCitedForInformationBy"
    IsCompiledBy = "IsCompiledBy"
    IsConfirmedBy = "IsConfirmedBy"
    IsCorrectedBy = "IsCorrectedBy"
    IsCreditedBy = "IsCreditedBy"
    IsCritiquedBy = "IsCritiquedBy"
    IsDeridedBy = "IsDeridedBy"
    IsDescribedBy = "IsDescribedBy"
    IsDisagreedWithBy = "IsDisagreedWithBy"
    IsDiscussedBy = "IsDiscussedBy"
    IsDisputedBy = "IsDisputedBy"
    IsDocumentedBy = "IsDocumentedBy"
    IsExtendedBy = "IsExtendedBy"
    IsLinkedToBy = "IsLinkedToBy"
    IsParodiedBy = "IsParodiedBy"
    IsPlagiarizedBy = "IsPlagiarizedBy"
    IsQualifiedBy = "IsQualifiedBy"
    IsRefutedBy = "IsRefutedBy"
    IsRetractedBy = "IsRetractedBy"
    IsReviewedBy = "IsReviewedBy"
    IsRidiculedBy = "IsRidiculedBy"
    IsSpeculatedOnBy = "IsSpeculatedOnBy"
    IsSupportedBy = "IsSupportedBy"
    IsUpdatedBy = "IsUpdatedBy"
    Likes = "Likes"
    LinksTo = "LinksTo"
    ObtainsBackgroundFrom = "ObtainsBackgroundFrom"
    ObtainsSupportFrom = "ObtainsSupportFrom"
    Parodies = "Parodies"
    Plagiarizes = "Plagiarizes"
    ProvidesAssertionFor = "ProvidesAssertionFor"
    ProvidesConclusionsFor = "ProvidesConclusionsFor"
    ProvidesDataFor = "ProvidesDataFor"
    ProvidesExcerptFor = "ProvidesExcerptFor"
    ProvidesMethodFor = "ProvidesMethodFor"
    ProvidesQuotationFor = "ProvidesQuotationFor"
    Qualifies = "Qualifies"
    Refutes = "Refutes"
    RepliesTo = "RepliesTo"
    Retracts = "Retracts"
    Reviews = "Reviews"
    Ridicules = "Ridicules"
    SharesAuthorInstitutionWith = "SharesAuthorInstitutionWith"
    SharesAuthorWith = "SharesAuthorWith"
    SharesFundingAgencyWith = "SharesFundingAgencyWith"
    SharesJournalWith = "SharesJournalWith"
    SharesPublicationVenueWith = "SharesPublicationVenueWith"
    SpeculatesOn = "SpeculatesOn"
    Supports = "Supports"
    Updates = "Updates"
    UsesConclusionsFrom = "UsesConclusionsFrom"
    UsesDataFrom = "UsesDataFrom"
    UsesMethodIn = "UsesMethodIn"
