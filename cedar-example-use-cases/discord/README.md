# Cedar Discord Example

This repository contains a limited model of the [discord permissions system](https://support.discord.com/hc/en-us/articles/206029707-Setting-Up-Permissions-FAQ).


The file `src/main.rs` sets up example users and demonstrates they have different permissions based on the different roles.
Discord is interesting because users may have multiple roles and some users may also set the permissions of other roles dynamically.
In this example, we implement this functionality by using
Cedar's parent system to build a DAG that looks something like this:

```
 Permission::"SendMessage"    Permission::"KickMember" 
         ▲        ▲                     ▲              
         │        └───────────────────┐ │              
         │                            │ │              
   Role::"everyone"              Role::"admin"         
         ▲                             ▲
         │                             │
   User::"yihong"                User::"oflatt"
```


We can then user Cedar's `in` construct to check if the permission
is reachable from a given user.
Note that it's currently unclear if this is the best way to use
Cedar for discord's permissions model. Another approach is to generate
many Cedar policies, one per role and permission pair.