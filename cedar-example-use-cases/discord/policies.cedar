// Allow SendMessage when the user has the SendMessage permission
permit (
  principal in Permission::"SendMessage",
  action == Action::"SendMessage",
  resource == Server::"test"
);

permit (
  principal in Permission::"KickMember",
  action == Action::"KickMember",
  resource == Server::"test"
);

permit (
  principal in Role::"owner",
  action == Action::"ManageRole",
  resource == Role::"everyone"
);