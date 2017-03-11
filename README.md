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
- API Cleanup: Move endpoints around under an `/api` and possibly doc the whole thing in a swagger spec.
- PubSub & Websockets: Create an event for notifications on changes to a session.

## Future Features
- Long term tracking & team spaces.
- Switch between points/hours, as well as average/totals.
