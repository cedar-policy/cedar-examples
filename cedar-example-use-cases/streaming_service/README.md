# Streaming service policies

This use-case simulates a streaming service to watch on-demand movies or shows.
The example makes use of the experimental `datetime` extension which requires feature flags to enable it (i.e., `--features datetime` when Cedar is being installed or compiled).

## Use-case

Members of the streaming service can be either `FreeMember` or `Subscriber`.
`Subscriber` is a paid membership with two tiers: `standard` and `premium`.
There are three main rules to grant access to content:
 - `subscriber-content-access/show`: A `Subscriber` can watch any `Show` as long it is not *Early Access*.
 - `subscriber-content-access/movie`: A `Subscriber` can watch any `Movie` as long it does not require to *Rent or Buy*.
 - `free-content-access`: A `FreeMember` can watch any content as long as it is free content.

The other three rules require the `datetime` extension to be encoded:
 - `rent-buy-oscar-movie`: Offers to *Rent or Buy* `Movie`s nominated to the Oscars during the month before the Oscars night. Makes use of the
 `datetime` constructor to build timestamps from datetime strings and compare against the current date and time.
 - `early-access-show`: Offers *Early Access* to watch `Show`s 24 hours before the official release date. Makes use of the `duration` constructor and the `offset` method to compare against release dates.
 - `forbid-bedtime-watch-kid-profile`: Forbids access to watch any content to kid profiles during bedtime. Makes use of the `duration` constructor and the `toTime` method to compare against the current time.

## Tests

The test setup defines the following entities, in `entities.json`:
 - Members: `Alice`, `Charlie` and `Dave` are `Subscribers`. `Bob` is a `FreeMember`. Only `Charlie` pays for the `premium` tier. The kid profile is enabled on `Dave`'s account.
 - Content: `The Godparent`, `The Gleaming`, `Devilish` are `Movie`s. `The Godparent` can be watched for free and was nominated to the Oscars. `Devilish` is also nominated to the Oscars. `Buddies` and `Breach` are `Show`s. `Breach` will be released on `2025-02-21` and available for *Early Access*.

We have created six scenarios.
Note that all requests except the last one use the same time: `2025-02-20T13:00:00-0500` (EST timezone offset).

1. `Alice` watches `Buddies` -- this is allowed per rule `subscriber-content-access/show`: `Alice` is a `Subscriber` so she can watch any
`Show` as long as it does not require *Early Access*.
2. `Bob` watches `The Godparent` -- this is allowed per rule `free-content-access`: `Bob` is a `FreeMember` and `The Godparent` is free content.
3. `Bob` watches `The Gleaming` -- this is denied because ``free-content-access` is not satisfied: : `Bob` is a `FreeMember` but `The Gleaming` cannot be watched for free.
4. `Alice` rents `Devilish` -- this is allowed per rule `rent-buy-oscar-movie`: `Alice` is a `Subscriber` and wants to rent the Oscar-nominated musical about 11 days before the Oscars.
5. `Charlie` watches `Breach` -- this is allowed per rule `early-access-show`: `Charlie` is a `Subscriber` on the `premium` tier so he can watch `Breach` through *Early Access*.
5. `Dave` watches `Buddies` at`22:00` -- this is not allowed per rule `forbid-bedtime-watch-kid-profile`: `Dave`'s account has the kid profile enabled and it is past bedtime.
