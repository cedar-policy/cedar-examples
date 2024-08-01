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
when { principal == resource.owner };

/// Users in the viewACL group can view the document unless it's private
@id("viewACL")
permit (
    principal,
    action == Action::"ViewDocument",
    resource
)
when { principal in resource.viewACL };

/// Any user can view a document when it's publicly accessible and not private
@id("public-view")
permit (
    principal,
    action == Action::"ViewDocument",
    resource
)
when { resource.publicAccess == "view" || resource.publicAccess == "edit" };

/// Users in the modifyACL group can modify the document unless it's private
@id("modifyACL")
permit (
    principal,
    action == Action::"ModifyDocument",
    resource
)
when { principal in resource.modifyACL };

/// Any user can modify a document when it's publicly editable and not private
@id("public-edit")
permit (
    principal,
    action == Action::"ModifyDocument",
    resource
)
when { resource.publicAccess == "edit" };

/// Users in the manageACL group can edit built-in groups
@id("manageACL")
permit (
    principal,
    action in [Action::"AddToShareACL", Action::"EditPublicAccess"],
    resource
)
when { principal in resource.manageACL };

/// Blocked users cannot perform these actions
@id("blocked-users")
forbid (
    principal,
    action in
        [Action::"ViewDocument",
         Action::"ModifyDocument",
         Action::"AddToShareACL",
         Action::"EditPublicAccess",
         Action::"DeleteDocument"],
    resource
)
when
{ resource.owner has blocked && resource.owner.blocked.contains(principal) };