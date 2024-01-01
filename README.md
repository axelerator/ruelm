# Rust + Elm = Rülm

Rülm is a minimal, fullstack web app template for Rust and Elm.
It is not a crate/library/framework (yet) but rather an app that's
supposed to be a starting point to build a full app.

The approach is inspired by [Lamdera's](https://dashboard.lamdera.app/docs) message exchange model.
That means the app developer does not have to care about the specifics of the underlying (http)protocol.

I you have questions or suggestions feel free reach out to me (@axelerator) on the [Elm Slack](https://elm-lang.org/community/slack)

## How it works

Directory structure:

```text
├── src
│   └── main.rs      // Rust server
├── client
│   └── src
│       └── Main.elm // Elm client 
└── www
    ├── index.html   // Entry point
    └── assets
        └── main.js  // compiled Elm code 
```

### Authentication

The index page contains a "hard coded" login form that sends a POST request to the `/login` endpoint.
If the credentials are correct, the server will respond with a redirect to the index page with
a `session_id` request parameter.

The `index.html` contains a small script that'll look for the `session_id` in the URL and then
load the Elm application with the `session_id` passed in as a flag.

### SSE (Server Sent Events)

If the login was successful, the `index.html` will also open a SSE connection using the `session_id`.

### Message exchange

There is one more route `/send` that the Elm client can use to send messages to the server.

Messages from the frontend to the backend are defined on the Rust side as the `ToBackend` enum.
When the server gets started it uses the [`elm_rs` crate]https://crates.io/crates/elm-rs) to write
the matching Elm types into the Elm applications `src` directory.

The same happens for the `ToFrontend` enum that defines the messages from the server to the frontend.

`ToBackend` messages can be sent using the `sendToBackend` function where they will be automatically
deserialized in the matching Rust type.

The messages will be processed in a single worked thread where the server can respond by sending
`ToFrontend` messages to one more clients (identified by `session_id`)

`ToFrontend` messages can be sent using the `sendToFrontend` function.

