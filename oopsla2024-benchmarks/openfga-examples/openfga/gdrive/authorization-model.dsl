model
  schema 1.1
type user
type group
  relations
    define member: [user]
type folder
  relations
    define can_create_file: owner
    define can_write_files: owner or can_write_files from parent
    define can_share_files: owner or can_share_files from parent
    define owner: [user]
    define parent: [folder]
    define viewer: [user,user:*,group#member] or owner or viewer from parent
type doc
  relations
    define owner: [user]
    define parent: [folder]
    define viewer: [user,user:*,group#member]
    define can_change_owner: owner
    define can_read: viewer or owner or viewer from parent
    define can_write: owner or can_write_files from parent
    define can_share: owner or can_share_files from parent
