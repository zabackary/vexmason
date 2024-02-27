# VEXMason

> _"If you can think of a better name, let me know."_

VEXMason is a comprehensive build system for VEX V5 Python. It lets you
modularize and refactor your code into maintainable and modern OOP Python
classes and files within the V5 ecosystem.

VEXMason acts as a compiler/bundler for Python, like Rollup is for JS, and makes
it so you can write your code in different modules and rely on the tooling to
condense it into an uploadable single Python file. It also lets you do CPP-style
`#define` at compile-time!

VEXMason utilizes [`python-compiler`](https://github.com/zabackary/python-compiler)
to bundle the files, which I also wrote.

## Features

- [x] Integrate into the native VEX VSCode extension and bundle files behind-
      the-scenes.
- [x] Read a config file _and substitute defined constants_ (seems to be broken).
- [ ] _(in progress)_ Lets you select possible constant values from a UI.

## Installation

> [!NOTE]
> 
> VEXMason is only supported on Windows for now. The code is cross-platform,
> though, so as soon as I finish initial development I'll work on supporting
> Linux. Please let me know if you would like to try to build on OSX.

Installation is easy. Just head over to the
[GitHub releases page](https://github.com/zabackary/vexmason/releases/) and
download/run the `installer.exe` file associated with the latest release.
Windows will (probably) flag the file as "unsafe". If you trust me, you can
ignore it. If you don't, read the code and [compile it from the source](#Development)
yourself.

## Development

### Building

You'll need the Rust compiler (rustc) and Cargo installed to build this project,
as it's written in [Rust](https://www.rust-lang.org/). The easiest way to do
this is through [`rustup`](https://rustup.rs/).

Once you've done that, you can build the project with `cargo build` and/or run
the code with `cargo run --bin {binary name}`. There are three main binaries built:
the `vexcom` binary, the main binary ("`vexmason`"), and the installer
("`installer`").

### Steps to put the binaries in the right place

1. Run the installer. That's it. It will fetch the binaries from GitHub, though, so|
   make sure you trust me.

or

1. Navigate to `~/.vscode/extensions/vexcode-{version}/resources/tools/vexcom/{platform}`
   and rename the existing `vexcom` or `vexcom.exe` to `vexcom.old` or
   `vexcom.old.exe`. Tip: make a backup and name it something like `vexcom.bak`
   beforehand in case something goes wrong.
2. Take your built `vexcom.exe` file (the one you built in [Building](#Building))
   and make a copy of it in that directory.
3. Create the installation directory at `%LOCALAPPDATA%/vexmason` and `git clone`
   [`python-compiler`](https://github.com/zabackary/python-compiler) into
   `./lib/python-compiler`.
4. Make a copy of your built `vexmason.exe` and put it in `./bin`.
