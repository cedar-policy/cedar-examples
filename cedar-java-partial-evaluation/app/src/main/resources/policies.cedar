@id("policy0")
permit(
    principal in UserGroup::"Admin",
    action in Action::"Read",
    resource in DocumentGroup::"Protected"
);

@id("policy1")
permit(
    principal in UserGroup::"Admin",
    action == Action::"Write",
    resource in DocumentGroup::"Protected"
) when {
    context.authenticated == true
};

@id("policy2")
forbid(
    principal == User::"Bob_admin",
    action == Action::"Read",
    resource == Document::"protected_doc1"
);

@id("policy3")
permit(
    principal == User::"Alice_admin",
    action == Action::"Read",
    resource == Document::"confidential_doc1"
);

@id("policy4")
permit(
    principal == User::"Alice_admin",
    action == Action::"Write",
    resource == Document::"confidential_doc1"
) when {
    context.authenticated == true
};

@id("policy5")
permit(
    principal == User::"Alice_admin",
    action == Action::"Read",
    resource == Document::"confidential_doc3"
);

@id("policy6")
permit(
    principal == User::"Alice_admin",
    action in Action::"Write",
    resource == Document::"confidential_doc3"
) when {
    context.authenticated == true
};

@id("policy7")
permit(
    principal == User::"Bob_admin",
    action == Action::"Read",
    resource == Document::"confidential_doc2"
);

@id("policy8")
permit(
    principal == User::"Bob_admin",
    action == Action::"Write",
    resource == Document::"confidential_doc2"
) when {
    context.authenticated == true
};


@id("policy9")
permit(
    principal is User,
    action == Action::"Read",
    resource in DocumentGroup::"Public"
);

@id("policy10")
permit(
    principal is UserGroup,
    action == Action::"Read",
    resource in DocumentGroup::"Public"
);

@id("policy11")
permit(
    principal in UserGroup::"Admin",
    action == Action::"Write",
    resource in DocumentGroup::"Public"
) when {
    context.authenticated == true
};

@id("policy12")
permit(
    principal == User::"John_customer",
    action == Action::"Read",
    resource == Document::"protected_doc1"
) when {
    context.authenticated == true
};

@id("policy13")
permit(
    principal == User::"Mark_customer",
    action == Action::"Read",
    resource == Document::"protected_doc2"
) when {
    context.authenticated == true
};

@id("policy14")
permit(
    principal == User::"Alice_admin",
    action == Action::"Delete",
    resource == DocumentGroup::"Public"
) when {
    context.authenticated == true &&
    context.delete_confirmation == true
};

@id("policy15")
permit(
    principal == User::"Bob_admin",
    action == Action::"Delete",
    resource == DocumentGroup::"Protected"
) when {
    context.authenticated == true &&
    context.delete_confirmation == true
};
