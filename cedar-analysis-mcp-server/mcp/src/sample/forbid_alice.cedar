// This policy explicitly forbids a specific user (alice@example.com) from commenting on any photos
@id("forbid-alice-comment")
forbid (
    principal is PhotoSharing::User,
    action == PhotoSharing::Action::"comment",
    resource is PhotoSharing::Photo
)
when {
    principal.email == "alice@example.com"
};