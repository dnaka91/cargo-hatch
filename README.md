# 🐣 Cargo Hatch

Hatch new projects like a chick coming out of its egg.

Cargo hatch is a `cargo init`/`cargo new` on steroids, allowing complex templates thanks to the
[Tera] engine. Additional template values can be configured and requested from the user during
execution.

## Why not `cargo-generate` instead?

This project was born out of frustration with cargo-generate. Most and foremost it didn't work out
at all for me with SEGFAULTs and random Git errors, so until today I wasn't able to create a single
project from a template using `cargo-generate`.

Another few points of frustration with it that are done differently here:

- **Way too many emojis**. Some are totally fine and definitely help to visually quickly skim the
  output but I personally felt that there are emojis in every little corner, way to many for my
  taste.
  - **Cargo hatch** instead tries to keep the output simple and use colors here and there.
    Eventually you may discover an emoji here and there as well.
- **Difficult to test templates locally**. Cargo generate seems to focus extremely on Git repos and
  that is fine by itself for published templates but testing your templates before publish or the
  next commit seemed difficult (especially repos without initial commit don't seem to work).
  - **Cargo hatch** provides a `local` subcommand that simply takes any directory and treats it as
    template making it easy to test templates or have local private templates, Git or not.
- **Input values are too limited**. At the moment of writing there are only strings and lists
  available as types of user input for templates. That is very limited and there are use cases for
  boolean, numbers and other values as well.
  - **Cargo hatch** tries to provide a broader selection of input types, currently including
    booleans, integers, floating point numbers, strings and lists (of strings).
- **Liquid template engine**. I personally am not the biggest fan of the Liquid engine that is used
  in cargo generate. It is kind of similar to Jinja2-ish engines but not quite the same. This may
  be a big matter of taste but was a problem point for me.
  - **Cargo hatch** uses the [Tera] template engine instead, providing a very Jinja2-ish syntax and
    if you use the Zola static-site-generator, you may already be familiar with it.

[Tera]: https://tera.netlify.app/

## Installation

To build this project have `rust` and `cargo` available in the latest version. `rustup` is the recommended way of installing and managing the Rust toolchain.

Then run the following command to install this project:

```sh
cargo install cargo-hatch
```

Make sure that your cargo binary path (usually `$HOME/.cargo/bin`) is available from your `$PATH`.

## Usage

The usage of `cargo-hatch` is rather detailed and therefore lived in a separate file. Check out
[USAGE.md](USAGE.md) for further instructions.

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
