// Allow SendMessage when the user has the SendMessage permission
permit (
  principal,
  action == Action::"SendMessage",
  resource == Server::"test"
)
when { principal in Permission::"SendMessage" };

permit (
  principal,
  action == Action::"KickMember",
  resource == Server::"test"
)
when { principal in Permission::"KickMember" };

permit (
  principal,
  action == Action::"ManageRole",
  resource == Role::"everyone"
)
when { principal in Role::"owner" };