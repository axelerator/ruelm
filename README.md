# Rust + Elm = Rülm

Rülm is a minimal, fullstack web app template for Rust and Elm.
It is not a crate/library/framework (yet) but rather an app that's
supposed to be a starting point to build a full app.
In less than 500 lines of code!

```text
$ wc -l src/main.rs client/src/Main.elm www/index.html
     183 src/main.rs
      95 client/src/Main.elm
      47 www/index.html
     325 total
```

The approach is inspired by [Lamdera's](https://dashboard.lamdera.app/docs) message exchange model.
That means the app developer does not have to care about the specifics of the underlying (http)protocol.

I you have questions or suggestions feel free reach out to me (@axelerator) on the [Elm Slack](https://elm-lang.org/community/slack)

## How to run

In the project directory:

```
~ ruelm $ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.22s
     Running `target/debug/ruelm`
```
This will generate the `Bindings.elm` module necessary to compile the Elm
client and starts the server. `Ctrl + C` to kill the server.

Change into the Elm client directory and compile it into the `www` folder like so:
```
~ ruelm » cd client
~ ruelm/client » elm make src/Main.elm --output=../www/assets/main.js
Success!

    Main ───> ../www/assets/main.js
```
Alternatively to the last step, there is also [a watch script](client/bin/watch.sh) that will recompile the Elm client on changes, but it's only tested on MacOs so far.

## How it works

Directory structure:

```text
├── src
│   └── main.rs              // Rust server
├── client
│   ├── src
│   │   ├── generated
│   │   │   └── Bindings.elm // generated Elm bindings
│   │   └── Main.elm         // Elm client 
│   └── bin
│       └── watch.sh         // (MacOs) watch script to compile on save
└── www
    ├── index.html           // Entry point
    └── assets
        └── main.js          // compiled Elm code 
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
When the server gets started it uses the [`elm_rs` crate](https://crates.io/crates/elm-rs) to write
the matching Elm types into the Elm applications `src` directory.

The same happens for the `ToFrontend` enum that defines the messages from the server to the frontend.

`ToBackend` messages can be sent using the `sendToBackend` function where they will be automatically
deserialized in the matching Rust type.

The messages will be processed in a single worker thread where the server can respond by sending
`ToFrontend` messages to one more clients (identified by `session_id`)

These messages get transferred to the Elm client through a port and then centrally processed
by the [`updateFromBackend` function](https://github.com/axelerator/ruelm/blob/main/client/src/Main.elm#L55)
