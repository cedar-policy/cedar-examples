// Here, ?principal is a group of users allowed to view ?resource, i.e., resource.viewers above
@createPolicyWhen("Create a template linked policy
                   when a external user is added to a prez as viewer")
@id("external-prez-view")
permit(principal == ?principal,
  action in Action::"ExternalPrezViewActions",
  resource == ?resource)
when {
  principal.job != Job::"internal"
};

// Here, ?principal is a group of users allowed to view ?resource
@createPolicyWhen("Create a template linked policy
                   when a internal user is added to a prez as viewer")
@id("internal-prez-view")
permit(principal == ?principal,
  action in Action::"InternalPrezViewActions",
  resource == ?resource)
when {
  principal.job == Job::"internal"
};

// Here, ?principal is a group of users allowed to edit ?resource, i.e., resource.editors above
@createPolicyWhen("Create a template linked policy
                   when a user is added to a prez as editor")
@id("template-edit for non-owner")                   
permit(
  principal == ?principal,
  action in Action::"PrezEditActions",
  resource == ?resource);

// Presentation owners always allowed to do what they want
@id("template-edit for owner")
permit(
  principal,
  action in Action::"PrezEditActions",
  resource)
when {
  resource.owner == principal
};

// only permit sharing to non-customers
@id("limit-prez-view-customer")
forbid(
  principal,
  action == Action::"grantViewAccessToPresentation",
  resource)
unless {
  context.target.job != Job::"customer" ||
  (principal.job == Job::"distributor" &&
   principal.customerId == context.target.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-prez-edit-to-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToPresentation",
  resource)
when {
  context.target.job != Job::"internal"
};


// TEMPLATE POLICIES
@createPolicyWhen("Create a template linked policy
                   when a user is added to a template as market viewer")
@id("market-template-view")
permit(
  principal == ?principal,
  action in Action::"MarketTemplateViewActions",
  resource == ?resource)
when {
  principal.job != Job::"internal"
};


@createPolicyWhen("Create a template linked policy
                   when a internal user is added to a template as viewer")
@id("internal-template-view")
permit(
  principal == ?principal,
  action in Action::"InternalTemplateViewActions",
  resource == ?resource)
when {
  principal.job == Job::"internal"
};

// Authorizes edit actions generally, but these limited with forbid policies
@id("template-edit")
@createPolicyWhen("Create a template linked policy
                   when a user is added to a template as editor")
permit(
  principal == ?principal,
  action in Action::"TemplateEditActions",
  resource == ?resource);

// Permit owners to edit templates 
permit(
  principal,
  action in Action::"TemplateEditActions",
  resource)
when {
  principal == resource.owner 
};

// only permit sharing by internal users to non-customers
@id("limit-template-grant-view")
forbid(
  principal,
  action == Action::"grantViewAccessToTemplate",
  resource)
when {
  context has targetUser && context.targetUser.job == Job::"customer" &&
  (principal.job != Job::"distributor" ||
   principal.customerId != context.targetUser.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-template-grant-edit-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToTemplate",
  resource)
when {
   context has targetUser && context.targetUser.job != Job::"internal"
   // context.targetMarket always Ok, no matter the market
};