permit (
    principal,
    action in [Action::"CreateList", Action::"GetLists"],
    resource == Application::"TinyTodo"
);

permit (principal, action, resource)
when {
    resource has owner &&
    resource.owner == principal
};

permit (
    principal,
    action == Action::"GetList",
    resource
)
when {
    principal in resource.readers ||
    principal in resource.editors
};

permit (
    principal,
    action in [
        Action::"UpdateList",
        Action::"CreateTask",
        Action::"UpdateTask",
        Action::"DeleteTask"
    ],
    resource
)
when { principal in resource.editors };

// permit (
//    principal in Team::"Admin",
//    action,
//    resource in Application::"TinyTodo"
// );
