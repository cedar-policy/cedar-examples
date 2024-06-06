# Cedar Discord Example

This repository contains a model of the [discord permissions system](https://support.discord.com/hc/en-us/articles/206029707-Setting-Up-Permissions-FAQ).


In brief, the discord permission system has a notion of "Roles",
which are sets of permissions.
Users can have multiple Roles.
Roles can allow or disallow permissions, such as sending messages.

Adding more complexity, Discord allows users to configure
the permissions associated with a role for a particular channel.
Not only does discord store the permissions associated with a role,
but also the permissions associated with that role per channel (ChannelRole).


In this example, we implement this functionality by using
Cedar's parent system to build a DAG.
We then query reachability in the DAG using the `in` construct.

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