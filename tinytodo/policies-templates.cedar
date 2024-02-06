// Policy 0: Any User can create a list and see what lists they own
permit (
    principal,
    action in [Action::"CreateList", Action::"GetLists"],
    resource == Application::"TinyTodo"
);

// Policy 1: A User can perform any action on a List they own
permit (principal, action, resource)
when { resource is List && resource.owner == principal };

// Policy 2: Users who are members of [?principal] are readers of [?resource]
@id("reader-template")
permit (
    principal in ?principal,
    action == Action::"GetList",
    resource == ?resource
);

// Policy 3: Users who are members of [?principal] are editors of [?resource]
@id("editor-template")
permit (
    principal in ?principal,
    action in
        [Action::"GetList",
         Action::"UpdateList",
         Action::"CreateTask",
         Action::"UpdateTask",
         Action::"DeleteTask"],
    resource == ?resource
);

// Policy 4: Admins can perform any action on any resource
// @id("admin-omnipotence")
// permit (
//    principal in Team::"admin",
//    action,
//    resource in Application::"TinyTodo"
// );
//
// Policy 5: Interns may not create new task lists
// forbid (
//     principal in Team::"interns",
//     action == Action::"CreateList",
//     resource == Application::"TinyTodo"
// );
//
// Policy 6: No access if not high rank and at location DEF, 
// or at resource's owner's location
// forbid(
//     principal,
//     action,
//     resource is List
// ) unless {
//     principal.joblevel > 6 && principal.location like "DEF*" ||
//     principal.location == resource.owner.location
// };