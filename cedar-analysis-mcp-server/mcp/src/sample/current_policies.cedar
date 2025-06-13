// Basic policies for PhotoSharing application

// 1. Users can view any non-private photos
@id("view-public-photos")
permit (
    principal is PhotoSharing::User,
    action == PhotoSharing::Action::"view",
    resource is PhotoSharing::Photo
)
when { resource.private == false };

// 2. Users can view, comment on, and delete photos they own
@id("owner-full-access")
permit (
    principal is PhotoSharing::User,
    action in [PhotoSharing::Action::"view", PhotoSharing::Action::"comment", PhotoSharing::Action::"delete"],
    resource is PhotoSharing::Photo
)
when { resource.owner == principal };

// 4. Any user can create photos
@id("create-photos")
permit (
    principal is PhotoSharing::User,
    action == PhotoSharing::Action::"create",
    resource is PhotoSharing::Photo
);
