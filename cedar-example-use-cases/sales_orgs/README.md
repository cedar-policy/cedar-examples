# Sales orgs policies

ABC has a contracted sales force, and sales folk have access to resources based on their role and the type of resource.

## Use-case

ABC has one principal type, which is a `User`. Users are distinguished by their `job` (an attribute), which is an enumeration. The job can either be _internal_, or some other _external_ type, which includes _distributor_ and _customer_. A customer is assigned to a particular distributor if it shares the distributor's customer ID. Users can be a member of zero or more `Market`s.

ABC has two protected resources, `Presentation` and `Template` (unfortunate name clash). The creator of a resource is its `owner`, who is permitted to carry out any action on the resource. Other users are granted _direct_ access to a resource by being added to an ACL as one of two roles, _viewer_ or _editor_. A `User` can also be granted access to a `Template` via a `Market` the user is a member of, again as either a viewer or editor. In other words, a `Template` has viewer and editor users, directly, but also viewer and editor markets, which grant access to the users within them. The permissions gained by being a viewer/editor depend on whether the `User` in question has an internal or external job.

There are some rules limiting how access can be shared. Only _distributor_ `User`s can share with _customer_ `User`s, and in particular those with their customer ID. And only _internal_ `User`s can be granted editor access to resources. 

## Approaches

The `static/` directory contains policies and a schema for that encodes the _viewer_ and _editor_ relations on resources, both for presentations and templates, as `Set`-typed attributes `viewers` and `editors` on the resources.

The `templated/` directory uses Cedar templates instead. We drop the `viewers` and `editors` attributes and follow a simple pattern: Whenever you would add a `User` to _resource_`.viewers`, instead link a template with `?principal` as the user and `?resource` as the viewer. Do likewise for editors. And, do similarly with viewer/editor status of `Market`s on ABC (not Cedar) `Template` resources.

These are the only differences in the encodings.

## Examples

In each directory is a `run.sh` file that carries out three authorization examples asking whether a principal, either `User::"Alce"`, `User::"Bob"`, or `User::"Charlie"`, in that order, is allowed to `Action::"viewPresentation"` on `Presentation::"proposal"`. They all use the `entities.json` file, and the `templated/` policies also use the `linked` file that expresses two template links.

The answers are, respectively, `ALLOW`, `ALLOW`, and `DENY`, for the following reasons:
* Alice is the owner of the presentation
* Bob is an allowed viewer of the presentation. In the `static/` policies this fact is expressed in the `entities.json` file as part of the `Presentation::"proposal"` entity. In the `templated\` policies this fact is expressed via template links, expressed in the `linked` file
* Charlie is neither or these things (nor is he an editor of the presentation)
