// 1. Same company access to non-private photos
@id("same-company-allow-comment")
permit (
    principal is PhotoSharing::User,
    action in PhotoSharing::Action::"comment",
    resource is PhotoSharing::Photo
)
when { 
    principal.company == resource.owner.company
};


forbid(
    principal,
    action in PhotoSharing::Action::"comment",
    resource
)
when {
    resource.private || resource.filename like "*.gif"
};