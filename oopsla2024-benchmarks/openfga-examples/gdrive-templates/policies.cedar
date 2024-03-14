// Template to grant a user or group read access to a document or folder.
@id("template")
permit (
    principal in ?principal,
    action == Action::"read",
    resource in ?resource
);

// If a document is public, then anyone can read it.
permit (
    principal,
    action == Action::"read",
    resource
)
when { resource.isPublic };

// A document's owner (or owners of the parent folder) can read, write to,
// or share the document.
permit (
    principal,
    action in [Action::"read", Action::"write", Action::"share"],
    resource
)
when
{ resource in principal.ownedDocuments || resource in principal.ownedFolders };

// A document's owner can change the owner.
permit (
    principal,
    action == Action::"changeOwner",
    resource
)
when { principal.ownedDocuments.contains(resource) };

// A folder's owner can create documents.
permit (
    principal,
    action == Action::"createDocument",
    resource
)
when { principal.ownedFolders.contains(resource) };