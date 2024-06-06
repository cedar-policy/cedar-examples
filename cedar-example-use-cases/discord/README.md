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
We can then user Cedar's `in` construct to check if the permission
is reachable from a given user.
Note that it's currently unclear if this is the best way to use
Cedar for discord's permissions model. Another approach is to generate
many Cedar policies, one per role and permission pair.

Here is the DAG representing roles:

```
 Allow::"SendMessage"          Allow::"KickMember"
        ▲                                        
        │                                        
        │                                        
  Role::"everyone"              Role::"owner"    
        ▲   ▲                          ▲         
        │   └──────────────────────┐   │         
        │                          │   │         
 Roles::"yihong"               Roles::"oflatt"   
```

The "owner" role is special, and don't need to be connected
to each of the `Allow` permissions.

In addition, we build a dag representing channel-specific permissions.
In particular, in the `announcements` channel we forbid everyone from sending messages.
As usual, the "owner" role overrides this in the cedar policy.

```
Disallow::"SendMessage"                                                        
          ▲                                                                     
          │                                                                     
          │                                                                     
ChannelRole::"everyone-announcements"                                       
       ▲           ▲                                                  
       │           └─────────────────────────┐                        
       │                                     │                        
ChannelRoles::"yihong-announcements"       ChannelRoles::"oflatt-announcements" 
```

Each `User` has a `Roles` reference.
Each `User` and `Channel` combination has a `ChannelRoles` reference.
The cedar policy can then first check channel roles, which override normal roles.
It can then check normal roles when channel roles are not set.

