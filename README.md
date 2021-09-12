# 🐣 Cargo Hatch

Hatch new projects like a chick coming out of its egg.

Cargo hatch is a `cargo init`/`cargo new` on steroids, allowing complex templates thanks to the
[Tera] engine. Additional template values can be configured and requested from the user during
execution.

## Why not `cargo-generate` instead?

This project was born out of frustration with cargo-generate. Although a great project it didn't
work for me personally as I got SEGFAULTs and Git errors whenever I tried to generate a new project
with it.

The main differences are:

- **Less emojis**. Cargo hatch tries to keep the output simple and use colors here and there.
  Eventually you may discover an emoji here and there as well.
- **Local templates**. Cargo hatch provides a `local` subcommand that simply takes any directory
  and treats it as template making it easy to test templates or have local private templates, Git
  or not.
- **More argument types**. Cargo hatch tries to provide a broader selection of input types,
  currently including booleans, integers, floating point numbers, strings and lists (of strings).
- **Tera template engine**. Cargo hatch uses the [Tera] template engine, providing a very
  Jinja2-ish syntax and if you use the Zola static-site-generator, you may already be familiar with
  it.

[Tera]: https://tera.netlify.app/

## Installation

To build this project have `rust` and `cargo` available in the latest version. `rustup` is the recommended way of installing and managing the Rust toolchain.

Then run the following command to install this project:

```sh
cargo install cargo-hatch
```

Make sure that your cargo binary path (usually `$HOME/.cargo/bin`) is available from your `$PATH`.

## Usage

The usage of `cargo-hatch` is rather detailed and therefore lives in a separate file. Check out
[USAGE.md](USAGE.md) for further instructions.

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
