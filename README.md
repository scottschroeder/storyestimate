# Estimate


## API

* POST `/session`: Create a new session
* GET `/session/:id`: Lookup all associated data with a session
* PATCH `/session/:id`: Update the `state` (in the body) to either `Reset`, `Vote`, or `Visible` _(Maybe we should work on those names)_
* DELETE `/session/:id`: Delete a session and any users in the session

* POST `/session/:id/user`: Create a new User, the name is specified by the `name` field in the body.
* PATCH `/session/:id/user/:name`: Place a vote, the vote is specified by the `vote` field in the body.
* DELETE `/session/:id/user/:name`: Delete a user


## Major TODOs
- 5xx -> 4xx: Many errors which should be 4xx codes are coming out as 5xx currently.
- Authorization: When POSTs are made, clients get a token for further use. We are not currently doing any auth.
- Refactor: We instantiate a new redis client on every call.
- API Cleanup: Move endpoints around under an `/api` and possibly doc the whole thing in a swagger spec.
- PubSub & Websockets: Create an event for notifications on changes to a session.
- Reap Sessions: If users don't helpfully cleanup, we should have timeouts on redis or something.
