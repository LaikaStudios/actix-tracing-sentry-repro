An actix-web app using a combination of

- https://crates.io/crates/actix-web
- https://crates.io/crates/actix-web-requestid
- https://crates.io/crates/tracing
- https://crates.io/crates/tracing-actix-web
- https://crates.io/crates/tracing-subscriber
- https://crates.io/crates/sentry
- https://crates.io/crates/sentry-actix


Roughly patching together a subscriber setup from a combination of the docs/readmes
for `tracing`, `tracing-actix-web`, and `sentry` (specifically `sentry-tracing`)
it appears there's some sort of deadlock when the sentry client tries to send
events home.

Running without a sentry dsn seems to work, but curling any of the endpoints when
it is will hang your terminal.

To run: 

```
# No sentry dsn configured
$ cargo run
# or with the sentry dsn:
$ SENTRY_DSN=******** cargo run
```

Then, take your pick of:

```
$ curl http://localhost:7878/panic
$ curl http://localhost:7878/err
$ curl http://localhost:7878/event
```