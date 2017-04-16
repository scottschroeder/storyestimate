# Estimate

## Testing

### Unit tests
```
cargo test
```

### Blackbox System Tests
The python tests run against a working http server. The default server is `http://localhost:8000` which matches what you get when you run `cargo run`.

The tests require `pytest` which can be installed with:
```bash
pip install pytest
```

You can invoke the tests like so:
```bash
cd systest
pytest -v
pytest -v --url https://storyestimates.org --port 443
```


## Major TODOs
- User renames should update session ID
- PubSub & Websockets: Create an event for notifications on changes to a session.

## Future Features
- Long term tracking & team spaces.
- Switch between points/hours, as well as average/totals.
