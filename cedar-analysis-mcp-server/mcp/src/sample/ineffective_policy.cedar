@id("same-company-private-photo-access")
permit (
    principal is PhotoSharing::User,
    action == PhotoSharing::Action::"view",
    resource is PhotoSharing::Photo
)
when {
    // User is from the same company as the photo owner AND the photo is private
    (principal == resource.owner) &&
    (resource.private == true)
};
