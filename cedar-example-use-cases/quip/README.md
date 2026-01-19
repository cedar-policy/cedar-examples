# Quip Policies
Quip is a service thatcombines documents, spreadsheets, chat, into a powerful collaboration platform that integrates with Salesforce. Run `.\run.sh` in order to see the example authz requests for Quip.

## Basic Model

Quip has one principal type, which is a `User`. Users have a unique `id`, a `name`, a boolean value for if they are `disabled` or not, as well as an optional `shared_folder_ids` attribute that contains a set of all folders shared with them.

Quip has two resources, `Threads` and `Folders`. 

A `Thread` has a unique `id`, an `author_id` of `User` type, and a `title`. They also have an optional set of Users under `user_ids` that are the individuals a document is shared with directly and not rhouh a folder. Threads have an optional `shared_folder_ids`, for all folders that exist in the shared list. To see the expanded list of users that are members of these shared folders, we have an optional `expanded_user_ids` attribute. The most important attribute here is the `access_levels` attribute, which is a set of `member_ids` as `User` types and their `level_of_access`. Finally, there is an optional parent_folder attribute, that contains the parent `Folder` entity (if there is one) of a thread.

A `Folder` has a unique `id`, a `title`, a `creator_id` of `User` type, and a `member_ids` attribute which contains a set of all `User`s who are members of a folder.


There are four levels of access a principal can have for a `Thread`. Those include `FullAccess`, `Edit`, `View`, and `Comment`. For a `Folder` a `User` can only have `FullAccess` or no access. Specific actions are further defined in the cedar schema.



## Examples
There are a few policies defined in `quip.cedar`, as well as sample entities defined in `entity.json`. The `run.sh` script runs multiple queries to confirm expected behavior for this data. To summarize the information in `entity.json`:

### Entities 
There are three `Users` (principals):

* Alice Smith
* Bob Johnson
* Charlie Brown


As well as two documents (`Threads`) and a `Folder` they are in.

* [Document] Budget Report
* [Document] Meeting Minutes
* [Folder] Administration


And then another document that is not in any folder 

* [Document] Charlie’s Personal Notes

### Assumptions

* Alice Smith is an administrator, so she has access to the entire Administration folder as a member.

* Bob Johnson is a part of the finance department, so he has individual comment access to the Budget Report, but no other documents in the folder.

* Charlie Brown is an intern with no access to the Administration folder or documents in it. But he created the a Personal Notes document.
