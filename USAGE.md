# Cargo Hatch usage instructions

This document describes the functions provided by the cargo-hatch binary as well as details about the templating engine and how to query the user for input.

## `cargo-hatch`

The `cargo-hatch` binary is the main way of generating new projects from templates in various ways.
Running `cargo hatch help` should give a good overview of the possible commands and should be mostly
self-explanatory.

Additionally, all possible commands are explained in the following sub-sections as well.

### `init`

**Currently not implemented yet!**

Initialize a new template by first invoking `cargo init`/`cargo new` to generate the default project
structure and then adding default `.hatch.toml` and `.hatchignore` files with comments that explain
how to configure the template.

Possible arguments are:

- `name` (optional): The name of the template which at the same time defines the folder it is
  generated in. It can be omitted to use the current folder as target and the project name is
  derived from the folder name.

### `list`

List all known bookmarks to be used in `cargo hatch new`, with the respective name and description.

### `new`

Create a new project from a template defined in the global settings.

Possible arguments are:

- `bookmark`: Name of the bookmarked template to use.
- `name` (optional): Name of the new project and target folder where it is generated in. It can be
  omitted to use the current folder as target, deriving the project name from the folder name.

### `git`

Create a new project from a template located in a remote Git repository.

Possible arguments are:

- `url`: HTTP or Git URL of the repository.
- `name` (optional): Name of the new project and target folder where it is generated in. It can be
  omitted to use the current folder as target, deriving the project name from the folder name.

Possible options are:

- `folder` (optional): Sub-folder within the repository that contains the template. Helpful if a
  single repository contains multiple templates.

### `local`

Create a new project from a template located in the local file system.

Possible arguments are:

- `path`: Local path to the template.
- `name` (optional): Name of the new project and target folder where it is generated in. It can be
  omitted to use the current folder as target, deriving the project name from the folder name.

### `completions`

Generate shell completions for cargo-hatch. Content is written to the standard output and won't
write to your shell config files directly. The output can be redirected to a file by the user.

Possible arguments are:

- `shell`: The shell to generate completions for. Can be one of `zsh`, `bash`, `fish`, `powershell`
  or `elvish`.

For example, to configure completions for the `fish` shell run the following:

```sh
cargo hatch completions fish > ~/.config/fish/completions/cargo-hatch.fish
```

## Global configuration

Cargo hatch has a global config file that allows to further adjust it to your needs on a device
level. That means it contains settings not specific to a template but to your usage of the binary.

### Git

Most of the git configuration is taken from your default git settings file on your device. Some
extra settings are available under this category.

- `ssh_key`: The SSH key to use for authentication, defined as a file system path. If the path is
  relative, it's considered relative to your home folder.
  - Cargo hatch will default to use your `ssh-agent` to get the right authentication key, if this
    setting is omitted.

For example, using an Ed25519-based SSH key:

```toml
[git]
ssh_key = ".ssh/id_ed25519"
```

### Bookmarks

Similar to how your browser uses bookmarks to allow for shortcuts to often used sites, cargo hatch
allows to save shortcuts to commonly used templates.

Each entry is defined with a `[bookmarks.<name>]` key where the `<name>` defines the name of it. The
mandatory fields are:

- `repository`: Git location of the template. Must be a **HTTP or Git URL**.
  - HTTP URL example: `https://github.com/dnaka91/awesome-template.git`.
  - Git URL example: `git@github.com:dnaka91/awesome-template.git`.

Additionally, the optional values fields are:

- `description`: Short description of the template that is shown when listing known bookmarks.
- `folder`: A sub-folder within the repository for use with mono-repos that contain multiple
  templates. If this is set, only the sub-folder is used as template root, ignoring the rest of the
  repository.

For example:

```toml
[bookmarks.server]
repository = "git@github.com:dnaka91/rust-server-template.git"
description = "Basic template for a web-server using the `axum` crate"
```

## Template configuration with `.hatch.toml`

The `.hatch.toml` file is the main configuration point of a template and **required** as a marker
for the repository (even if it's empty). The file itself is excluded from the generated files.

### Project name

The project name is taken from `new`/`git`/`local` commands, either provided on the CLI or derived
from the current directory. It is globally available in each template through the `project_name`
variable.

### Crate type

Key: `crate_type`

The crate type is either `bin` or `lib`, exactly as with `cargo init`/`cargo new`. By default the
type is queried from the user during execution but depending on the template it makes sense to fix
the type instead. If this value is set it wont be asked during execution.

For example a template for web servers is likely always a `bin` crate:

```toml
crate_type = "bin"
```

Based on this value, 3 pre-defined values are inserted into every templates context:

- `crate_type` (string): Either `bin` or `lib`, same as the setting.
- `crate_bin` (bool): `true` if the crate type is `bin`, `false` otherwise.
- `crate_lib` (bool): `true` if the crate type is `lib`, `false` otherwise.

For example, the following template:

```jinja2
Crate type: {{ crate_type }}
Is bin:     {{ crate_bin }}
Is lib:     {{ crate_lib }}
```

Would render to the following, given that `bin` was selected for the `crate_type`:

```txt
Crate type: bin
Is bin:     true
Is lib:     false
```

### Git information

Additional git information is loaded from the device's Git configuration files and put into the
context of each template. The provided values are:

- `git_author`: The Git user name and email combined as it was used by `cargo` until it was marked
  deprecated. The format is `name <email>`.
- `git_name`: User name as defined in the git config's `user.name` field.
- `git_email`: User email as defined in the git config's `user.email` field.

### Ignore paths

Key: `ignore`

Sometimes you want to ignore certain files and folders based on the arguments selected by the user.
For example some files may only be needed for a library or binary crate. You can do so by using the
`[[ignore]]` settings, which allow you to use the available template variables to decide upon file
exclusion.

You can create as many ignore settings as you like. Each requires a list of **glob patterns** to
select the files to exclude, and a condition to decide whether to actually exclude these files.

- `paths`: List of **glob patterns** to select excluded files.
- `condition`: Tera template that is executed to determine exclusion. It is filled with all the
  pre-defined and user-selected variables, same as the regular templates. It must evaluate to either
  `true` or `false`.

The following is a basic example, that excludes the `src/main.rs` when creating a library crate, and
excludes CLI related Rust files and shell completions when the selected `kind` is a web server.

```toml
[[ignore]]
paths = ["src/main.rs"]
condition = "{{ crate_lib }}"

[[ignore]]
paths = ["src/cli/**", "completions/**"]
condition = '{{ kind == "webserver" }}'
```

### Arguments

Arguments are extra values that are queried from the user to render the template and specific to
each individual template project. As the bare minimum, each argument must provide the following
settings:

- `type`: To define what kind of argument is used. One of `bool`, `string`, `number`, `float` or
  `list`.
- `description`: Short description of the value being asked for and printed out to the user when
  being prompted for a value.

Additional optional settings are:

- `default`: Define a default value that is pre-selected when the user is prompted for input. The
  value must match the type of the argument.

The `name` of each argument is its key in the settings file. See the following sub-sections for
examples of how to define the arguments.

#### Booleans

Booleans are simple binary `true`/`false` values, like the `bool` type in Rust.

```toml
[happy]
type = "bool"
description = "Are you happy?"
default = true
```

```txt
Are you happy? [Yn]:
```

#### Strings

Strings are the most generic argument and allow for any valid UTF-8 content.

```toml
[name]
type = "string"
description = "What's your name?"
```

```txt
What's your name?:
```

#### String validators

In addition to free-form strings arguments, a validator can bespecified, that further restrains the
allowed input.

##### Crate

Value: `crate`

The crate validator restricts the input to values that are considered a proper Rust crate name,
according to the [crates.io](https://crates.io) rules. That means precisely:

- Starts with an [alphabetic] character.
- All following characters are [ASCII alphanumeric], `_` or `-`.
- The value is only up to **64** characters long.

[alphabetic]: https://doc.rust-lang.org/std/primitive.char.html#method.is_alphabetic
[ASCII alphanumeric]: https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_alphanumeric

```toml
[crate_name]
type = "string"
description = "Name a crate"
validator = "crate"
```

##### Rust identifier

Value: `ident`

The ident validator ensures, the input is a Rust identifier. That means it adheres to the following
rules:

- Starts with a [Unicode XID Start] character, and has optional following characters, that are all
  in the [Unicode XID Continue] space.
- Or starts with an underscore `_` and is followed by one or more [Unicode XID Continue] characters.
- Is not a reserved [Rust keyword].

[Unicode XID Start]: https://docs.rs/unicode-ident/1.0.0/unicode_ident/index.html
[Unicode XID Continue]: https://docs.rs/unicode-ident/1.0.0/unicode_ident/index.html
[Rust keyword]: https://docs.rs/check_keyword/0.2.0/check_keyword/index.html

```toml
[const_name]
type = "string"
description = "Please provide a name for the constant"
validator = "ident"
```

##### Semantic version

Value: `semver`

The semver validator verifies, that the input value is a proper semantic version. It means, it must
have the typical `X.Y.Z` form and optional prerelease or build metadata values.

See the [semver::Version](https://docs.rs/semver/1.0.9/semver/struct.Version.html) for further
details about requirements for a valid value.

```toml
[version]
type = "string"
description = "What version do you want to use?"
validator = "semver"
```

##### Semantic version requirement specification

Value: `semver_req`

This validator is very similar to the semver validator, but instead allows version requirements,
rather than a singular version. It the same content that is specified in the `Cargo.toml` for
dependency versions.

See the [semver::VersionReq](https://docs.rs/semver/1.0.9/semver/struct.VersionReq.html) for further
details.

```toml
[crate_version]
type = "string"
description = "What version do you want to use?"
validator = "semver_req"
```

##### Regular expression

Value: `regex`

The regex validator allows for any regular expression to be used as input validation. Please note
that the value is defined slightly different for this validator, as it has to define the validator
type **and** the actual regex.

**Note:** Usually it is recommended to use `^` and `$` at the beginning and end of the regex, as it
ensures to check against the **whole** input. Otherwise, the input may only be partially checked and
empty values may become possible.

```toml
[name]
type = "string"
description = "Please provide a name, only in lowercase letters"
validator.regex = "^[a-z]+$"
```

#### Numbers

Numbers are parsed as 64-bit integers (Rust's `i64` type) and can optionally define a valid minimum
and maximum value. The boundaries are both defined as inclusive, which means up to the value
including the value itself. For example a maximum of `50` would consider the input `50` valid as
well.

- `min` (optional): Defines the minimum inclusive possible value.
- `max` (optional): Defines the maximum inclusive possible value.

```toml
[age]
type = "number"
description = "How old are you?"
min = 0
max = 100
```

```txt
How old are you? (0..=100):
```

#### Floats

Floats work the same as numbers, but allow for 64-bit floating point values (Rust's `f64` type).
Again, minimum and maximum values can optionally be set and are inclusive, meaning the defined value
is considered a valid input as well.

- `min` (optional): Defines the minimum inclusive possible value.
- `max` (optional): Defines the maximum inclusive possible value.

```toml
[height]
type = "float"
description = "How tall are you (in cm)?"
min = 0.0
max = 300.0
```

```txt
How tall are you (in cm)? (0..=300):
```

#### Lists

Lists define a fixed set of possible input values. They're similar to Rust enums but defined as
string value. The default value **must** be in the list of possible values or an error will be
printed.

```toml
[animal]
type = "list"
description = "What's your favorite animal?"
values = [
    "cat",
    "dog",
    "fish",
]
default = "fish"
```

```txt
What's your favorite animal?:
  cat
  dog
* fish
```

#### Multi-lists

Multi-lists are very similar to normal lists but allow to pick multiple items at once. Again,
default values must all be part of the possible values or an error will be printed. Individual
elements can be (un-)selected with the spacebar or tab.

```toml
[features]
type = "multi_list"
description = "Which server features would you like to enable?"
values = [
    "auth",
    "compression",
    "graceful-shutdown",
    "logging",
]
default = [
    "graceful-shutdown",
    "logging",
]
```

```txt
Which server features would you like to enable?:
  [ ] auth
  [ ] compression
* [x] graceful-shutdown
  [x] logging
```

## Exclude files with `.hatchignore`

The `.hatchignore` file is identical to a `.gitignore` file and supports the same patterns. It
allows to exclude certain files and directories from the pipeline. The file itself is already
excluded and doesn't need to be added to the list of ignored files.

Considering the following layout:

```txt
docs
  index.md
  info.md
.hatch.toml
.hatchignore
Cargo.toml
```

And the `.hatchignore` with the following content:

```txt
/docs
```

The generated project would be:

```txt
Cargo.toml
```

`.hatch.toml` and `.hatchignore` are automatically excluded and the additional filter rules exclude
the `docs` folder and everything withing. Therefore, only the `Cargo.toml` remains.
