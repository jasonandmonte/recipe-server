# Recipe Server
Author: Jason Gonzales

## Overview

This project implements a web server that serves randomly selected recipes. Currently on startup recipes are read from a JSON file (`assets/static/recipes.json`) into memory. When a user accesses the home page or refreshes a random recipe is displayed.

## Setup & Development

```sh
cargo install sqlx-cli
mkdir db && sqlx database create --database-url sqlite://db/recipes.db
```

Make `.env` file with database path `DATABASE_URL=sqlite://db/recipes.db`

Create migrations:
```sh
sqlx migrate add -r -s <name>
```

Apply migration:
```sh
sqlx migrate run --database-url sqlite://db/recipes.db
```

First run to add recipes from `.json` file:
```sh
cargo run -- --init-from assets/static/recipes.json
```


`cargo run --release`: This will run the server on `http://127.0.0.1:3000`

## Testing

Below is an entry in the JSON data and database for testing.
```json
{
    "id": "test",
    "title": "Test Recipe",
    "ingredients": "1/2 an ingredient\n1 cup second ingredient",
    "instructions": "Combine first ingredient and second ingredient.\nCook until ready to serve.",
    "source": "test",
    "tags": ["test"]
}
```

## Notes

The favicon.ico was made using PowerPoint and the basic shapes tool to create a cooking pot.

### Tracing Resources

[Adding Logging & Tracing Overview](https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust)
[Tokio: Getting started with Tracing](https://tokio.rs/tokio/topics/tracing)
[EnvFilter Examples](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#examples)
