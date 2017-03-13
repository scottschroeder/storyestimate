# Estimate


## Major TODOs
- Refactor module structure, separate out front vs. back facing structs
- Association tables should be cleaned up
- User votes should either be tied to a session or session should clear user votes on join
  - User renames should update session ID
- PubSub & Websockets: Create an event for notifications on changes to a session.
- Eval mockrequest and the potential issues with redis
- Write python test client to run blackbox system tests

## Future Features
- Long term tracking & team spaces.
- Switch between points/hours, as well as average/totals.
