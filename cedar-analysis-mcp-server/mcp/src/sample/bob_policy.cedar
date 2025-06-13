// Bob can view non-private photos
@id("bob-view-public-photos")
permit (
    principal is PhotoSharing::User,
    action == PhotoSharing::Action::"view",
    resource is PhotoSharing::Photo
)
when { 
    principal.email == "bob@example.com" &&
    resource.private == false 
};
