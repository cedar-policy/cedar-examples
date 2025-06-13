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

// 2. Do not allow commenting on private photos or *.gif
@id("no-common-on-private-or-gif")
forbid(
    principal,
    action in PhotoSharing::Action::"comment",
    resource
)
when {
    resource.private || resource.filename like "*.gif"
};
