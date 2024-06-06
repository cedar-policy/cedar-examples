# Taxpreparer policies

This use-case simulates an organization that prepares taxes for its clients.

## Use-case

A `Professional` needs to access a `Client`'s `Document` in order to prepare their taxes. There are two rules that grant access:

1. That the `Professional` has been granted access to the document. This could be because 
    
    a. she belongs to an _organization_ whose features (service line, location, etc.) are a match for features of the `Document`, or 
  
    b. because she has been granted _ad hoc_ access.
2. That the `Client` has given consent for professionals in a particular country to look at documents they own. A consent is modeled as a `Consent` entity, which is passed in with the authorization request's `context`.

Rules 1a and 2 are expressed as static policies, and rule 1b is expressed as a link to a template.

Rules 2 is expressed as a `forbid` rule so that it affects any _ad hoc_ `permit` policies added later. Alternatively, it could have been expressed as an additional `when` clause in the `permit` policies, both the static one and the template.

## Tests

The test setup defines the following entities, in `entities.json`:

- `Professional`s `Alice` and `Bob`. They are both part of `org-1` in the `corporate` serviceline, and are located at `IAD` and `JFK`, respectively.
- `Client` `Ramon` is contracting with the `corporate` serviceline
- `Document`s `ABC` and `DEF`, owned by `Ramon`, and located at `IAD` and `JFK`, respectively.

The setup also includes file `linked`, which links the ad-hoc access template to grant `Alice` access to document `DEF`.

Then we have five scenarios:

1. Alice requests access to ABC -- this is allowed per rules 1a and 2: Alice is part of the appropriate serviceline, organization, and location, and the request shows that her particular location has been consented to
2. Alice requests access to DEF -- this is allowed per rules 1b and 2: She has been granted ad hoc access, and the request shows that her particular location has been consented to
3. Bob requests access to DEF -- this is allowed per rules 1a and 2.
4. Alice requests access to ABC -- this time the request is denied because rule 2 is not satisfied: the provided consent does not include Alice's location
5. Bob requests access to ABC -- this is not allowed because neither rules 1a nor 1b are satisfied.