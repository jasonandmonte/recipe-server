# Recipe Server
Author: Jason Gonzales

## Overview

This project implements a web server that serves randomly selected recipes. Currently on startup recipes are read from a JSON file (`assets/static/recipes.json`) into memory. When a user accesses the home page or refreshes a random recipe is displayed.

## Setup

`cargo run --release`: This will run the server on http://127.0.0.1:3000

## Notes

The favicon.ico was made using PowerPoint and the basic shapes tool to create a cooking pot.

### Tracing Resources

[Adding Logging & Tracing Overview](https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust)
[Tokio: Getting started with Tracing](https://tokio.rs/tokio/topics/tracing)
[EnvFilter Examples](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#examples)
