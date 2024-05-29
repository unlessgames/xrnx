<img src="https://www.renoise.com/sites/default/files/renoise_logo_0.png" alt="Renoise" height="100"/>

# Renoise Scripting Docs

This repository contains the documentation for tool scripting in Renoise.

You can read the docs at [https://unlessgames.github.io/xrnx](https://unlessgames.github.io/xrnx) or by browsing the `docs` directory here.

## Development

The docs are generated using [mdBook](https://github.com/rust-lang/mdBook). To preview the pages locally you will need [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install mdbook.

```sh
cargo install mdbook
```

Then you can serve the docs at `localhost:3000` using mdbook, this will automatically refresh the browser tab whenever you change markdown files.

```sh
cd xrnx
mdbook serve --open
```