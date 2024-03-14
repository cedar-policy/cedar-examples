model
  schema 1.1
type application
  relations
    define member: [user,team#member,user:*]
    define create_list: member
    define get_lists: member
type user
type team
  relations
    define member: [user,team#member]
type list
  relations
    define owner: [user]
    define editor: [user,team#member] or owner
    define reader: [user,team#member,user:*] or owner or editor
    define get_list: reader
    define update_list: editor
    define create_task: editor
    define update_task: editor
    define delete_task: editor
    define edit_shares: owner
    define delete_list: owner
