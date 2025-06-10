model
  schema 1.1
type user
type team
  relations
    define member: [user,team#member]
type repo
  relations
    define admin: [user,team#member] or repo_admin from owner
    define maintainer: [user,team#member] or admin
    define owner: [organization]
    define reader: [user,team#member] or triager or repo_reader from owner
    define triager: [user,team#member] or writer
    define writer: [user,team#member] or maintainer or repo_writer from owner
type organization
  relations
    define member: [user] or owner
    define owner: [user]
    define repo_admin: [user,organization#member]
    define repo_reader: [user,organization#member]
    define repo_writer: [user,organization#member]
