@id("thread_author_full_access")
permit(
    principal,
    action in [Action::"ThreadFullAccessActions"],
    resource
)
when {
    principal == resource.author_id
};

@id("folder_member_full_access")
permit(
    principal,
    action in [Action::"FolderFullAccessActions"],
    resource
)
when {
    resource.member_ids.contains(principal)
};

@id("thread_comment_access_if_user_access_level_comment")
permit(
    principal,
    action in [Action::"ThreadCommentActions"],
    resource
)
when {
    resource has access_levels &&
    resource.access_levels.contains(
        {
            "member_id": principal, 
            "level_of_access": "Comment"
        }
    )};


@id("thread_full_access_if_user_access_level_full")
permit(
    principal,
    action in [Action::"ThreadFullAccessActions"],
    resource
)
when {
        resource has access_levels &&

    resource.access_levels.contains(
        {
            "member_id": principal, 
            "level_of_access": "FullAccess"
        }
)};

@id("thread_edit_access_if_user_access_level_edit")
permit(
    principal,
    action in [Action::"ThreadEditActions"],
    resource
)
when {
        resource has access_levels &&

    resource.access_levels.contains(
        {
            "member_id": principal, 
            "level_of_access": "Edit"
        }
)};


@id("thread_view_access_if_user_access_level_view")
permit(
    principal,
    action in [Action::"ThreadViewActions"],
    resource
)
when {
        resource has access_levels &&

    resource.access_levels.contains(
        {
            "member_id": principal, 
            "level_of_access": "View"
        }
)};