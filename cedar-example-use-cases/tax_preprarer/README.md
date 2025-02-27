# Tax Preparer

In this example, ABC company prepares taxes for its clients.

For more information about exploring this example using the Cedar CLI, see [Cedar Example Use Cases](https://github.com/cedar-policy/cedar-examples/tree/release/4.0.x/cedar-example-use-cases).

## Use-case

A `Professional` needs to access a `Document` from a `Client` in order to prepare their taxes. There are two ways she could get access:

1. The `Professional` has been granted access to the document. This could be because (a) she belongs to an _organization_ whose features (service line, location, etc.) are a match for features of the `Document`, or (b) she has been granted _ad hoc_ access.
	
2. The `Client` has given consent for professionals in a particular country to look at documents they own. A consent is modeled as a `Consent` record, which is passed in with the authorization request's `context`.


Rule _2_ is expressed as a `forbid` rule so that it affects any _ad hoc_ `permit` policies added later. Alternatively, it could have been expressed as an additional `when` clause in the `permit` policies, both the static one and the template.

Rules _1a_ and _2_ are expressed as static policies, and rule _1b_ is expressed as a template-linked policy.

### Entities

#### `orgInfo`
Represents information about an organization to which a professional can be assigned.

#### `Professional`
Represents a user that is preparing taxes for clients.

#### `Client`
Represents a user whose taxes are being prepared.

#### `Document` 
Represents a document that is needed by a professional to prepare a client's taxes.

#### `Consent`
Represents that the client has give consent for the document to be viewed.

### Actions

#### `viewDocument`
View a document.

### Schema

#### Namespace
* `Taxpreparer`

#### Entity types
* `orgInfo`
  * Attributes:
    * `organization`: a String
	* `serviceline`: a String
	* `location`: a String 
* `Professional`:
  * Attributes:
    * `assigned_orgs`: a set of `orgInfo` objects
	* `location`: a String
* `Client`:
  * Attributes:
    * `organization`: a String
* `Document`:
  * Attributes
  	* `serviceline`: a String
	* `location`: a String 
    * `owner`: a `Client`
* `Consent`:
  * Attributes 
    * `client`: a `Client`
	* `team_region_list`: a set of String
	
#### Action types
* `viewDocument`
  * principals: `Professional`
  * resources: `Document`
  * context: `consent`

### Policies

```
// Rule 1a: organization-level access
permit (
  principal,
  action == Taxpreparer::Action::"viewDocument",
  resource
)
when
{
  principal.assigned_orgs
    .contains
    (
      {organization
       :resource.owner.organization,
       serviceline
       :resource.serviceline,
       location
       :resource.location}
    )
};

// Rule 1b: ad hoc access (per linked template)
@id("adhoc-access")
permit (
  principal == ?principal,
  action == Taxpreparer::Action::"viewDocument",
  resource == ?resource
);

// Rule 2: consent must be given by a document's owner
forbid (
  principal,
  action == Taxpreparer::Action::"viewDocument",
  resource
)
unless
{
  context.consent.client == resource.owner &&
  context.consent.team_region_list.contains(principal.location)
};
```

## Tests

We use the following entities for our tests, in the `entities.json` file:
* There are 2 `Professional` entities, `Taxpreparer::Professional::"Alice"` and `Taxpreparer::Professional"Bob"`. They are both part of `org-1` organization in the `corporate` serviceline, and are located at `IAD` and `JFK`, respectively.
* There is one `Client` entity, `Taxpreparer::Client::"Ramon"`. He is contracting with the `corporate` serviceline.
* There are 2 `Document` entities, `Taxpreparer::Document::"ABC"` and `Taxpreparer::Document::"DEF"`. They are both owned by Ramon, and located at `IAD` and `JFK`, respectively.

The setup also includes file `linked`, which links the ad-hoc access template to grant Alice access to document DEF.



Here are some authz requests to test, included in the `ALLOW` and `DENY` folders:
* Alice requests access to ABC: ALLOW per rules `1a` and `2`. Alice is part of the appropriate serviceline, organization, and location, and the request shows that her particular location has been consented to by Ramon.
* Alice requests access to DEF: ALLOW per rules `1b` and `2`. Alice has been granted ad hoc access, and the request shows that her particular location has been consented to by Ramon.
* Bob requests access to DEF: ALLOW per rules `1a` and `2`. Bob is part of the appropriate serviceline, organization, and location, and the request shows that his particular location has been consented to by Ramon.
* Alice requests access to ABC: DENY per rule `2`. The consent provided by Ramon does not include Alice's location.
* Bob requests access to ABC: DENY per rules `1a`. Even though Bob's location is consented to by Ramon, it doesn't match the location of the document.
