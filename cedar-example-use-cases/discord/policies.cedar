permit (
  principal,
  action == Action::"SendMessage",
  resource
)
when
{
  // the user's channel matches the resource
  (principal.channel == resource) &&
  // the channel-specific roles this user has allow it
  (principal.channel_roles in Allow::"SendMessage" &&
   !(principal.channel_roles in Deny::"SendMessage"))
};

permit (
  principal,
  action == Action::"SendMessage",
  resource
)
when
{
  // the channel of the user matches the resource
  (principal.channel == resource) &&
  // the channel roles don't mention this permission
  (!(principal.channel_roles in Allow::"SendMessage" ||
    principal.channel_roles in Deny::"SendMessage")) &&
  // the global roles give the user permission
  (principal.roles in Allow::"SendMessage") &&
  !(principal.roles in Deny::"SendMessage")
};

// Allow any action by the owner of the server
permit (
  principal,
  action,
  resource == Role::"everyone"
)
when { principal.roles in Role::"owner" };