/// A drive owner can perform any actions on it
@id("drive-owner")
permit (
    principal,
    action,
    resource is Drive
)
when { resource.owner == principal };

/// A document owner can perform any actions on it
@id("document-owner")
permit (
    principal,
    action,
    resource is Document
)
when { principal == resource.owner };

/// A group owner can perform any actions on it
@id("group-owner")
permit (
    principal,
    action,
    resource is Group
)
when { resource has owner && principal == resource.owner };

/// Users in the viewACL group can view the document
@id("viewACL")
permit (
    principal,
    action == Action::"viewDocument",
    resource
)
when { principal in resource.viewACL };

/// Any user can view a document when it's publicly accessible
@id("public-view")
permit (
    principal,
    action == Action::"viewDocument",
    resource
)
when { resource.publicAccess == "view" || resource.publicAccess == "edit" };

/// Users in the modifyACL group can modify the document
@id("modifyACL")
permit (
    principal,
    action == Action::"modifyDocument",
    resource
)
when { principal in resource.modifyACL };

/// Any user can modify a document when it's publicly editable
@id("public-edit")
permit (
    principal,
    action == Action::"modifyDocument",
    resource
)
when { resource.publicAccess == "edit" };

/// Users in the manageACL group can edit built-in groups
@id("manageACL")
permit (
    principal,
    action in Action::"addToShareACL",
    resource
)
when { principal in resource.manageACL };

/// Blocked users cannot perform any operations to the document
@id("blocked-users")
forbid (
    principal,
    action,
    resource is Document
)
when
{ resource.owner has blocked && resource.owner.blocked.contains(principal) };