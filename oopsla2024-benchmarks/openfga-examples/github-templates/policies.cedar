// Grant read access to a user, team, or organization.
@id("readTemplate")
permit (
    principal in ?principal,
    action == Action::"read",
    resource == ?resource
);

// Grant triage access to a user, team, or organization.
@id("triageTemplate")
permit (
    principal in ?principal,
    action in Action::"triage",
    resource == ?resource
);

// Grant write access to a user, team, or organization.
@id("writeTemplate")
permit (
    principal in ?principal,
    action in Action::"write",
    resource == ?resource
);

// Grant maintainer access to a user, team, or organization.
@id("maintainTemplate")
permit (
    principal in ?principal,
    action in Action::"maintain",
    resource == ?resource
);

// Grant admin access to a user, team, or organization.
@id("adminTemplate")
permit (
    principal in ?principal,
    action in Action::"admin",
    resource == ?resource
);

// Users also inherit permissions from the owner of a repository.
permit (
    principal,
    action == Action::"read",
    resource
)
when { principal in resource.owner.readers };

permit (
    principal,
    action in Action::"write",
    resource
)
when { principal in resource.owner.writers };

permit (
    principal,
    action in Action::"admin",
    resource
)
when { principal in resource.owner.admins };
