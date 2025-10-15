# Tonic + Protocheck + Svelte

This is a full example of how to set up a fullstack application where validation is done using [protovalidate](https://github.com/bufbuild/protovalidate) annotations, enforced by [protocheck](github.com/Rick-Phoenix/protocheck) on the backend and [protovalidate-es](github.com/bufbuild/protovalidate-es) on the frontend. 

## Backend

The backend consists of a server that uses **Tonic** to listen for grpc requests (with a `grpc-web` layer provided by **Tonic-web**), while using **Axum** to serve normal http REST endpoints.

[protocheck](github.com/Rick-Phoenix/protocheck) is used to validate incoming grpc requests.

## Frontend

The frontend consists of a very simple **Sveltekit** app that uses [protovalidate-es](github.com/bufbuild/protovalidate-es) to validate outgoing messages and [connect-es](https://github.com/connectrpc/connect-es) to send them via `grpc-web` to the backend.

It also uses [Tanstack Form](https://tanstack.com/form/v1/docs/framework/svelte/quick-start) to provide responsive feedback from validation errors.

## How To Test

1. Clone the repo
2. Run `cargo run` to start the server
3. Install js dependencies with `pnpm install`
3. Run `pnpm run dev` to start the client
4. Send a request from the client and check the validation outcome
