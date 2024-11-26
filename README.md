<div align="center">
      <img src="./assets/vexmason-logo-outlined.svg" alt="vexmason logo" width="240" height="240" />
</div>
<h1 align="center">vexmason</h1>

<table align="center">
      <tr>
            <td>
                  <b>
                        Get started now!
                        <i><a href="https://github.com/zabackary/vexmason-template">
                              Go make a copy of the vexmason template
                        </a></i>
                  </b>
            </td>
      </tr>
</table>

vexmason is a comprehensive build system for VEX V5 Python. It lets you
modularize and refactor your code into maintainable and modern OOP Python
classes and files within the V5 ecosystem.

vexmason acts as a compiler/bundler for Python, like Rollup is for JS, and makes
it so you can write your code in different modules and rely on the tooling to
condense it into an uploadable single Python file. It also lets you do CPP-style
`#define` at compile-time!

vexmason utilizes
[`python-compiler`](https://github.com/zabackary/python-compiler) to bundle the
files, which I also wrote.

Why "vexmason"? Mason means to someone/thing who builds something, and I didn't
have any better ideas.

> [!WARNING]
>
> ~~As the maintainer's team has switched to either PROS or vexide, this repo isn't
> maintained much anymore. No new features will be added but bugs will still be
> fixed if they are reported.~~
>
> One of the teams working with the maintainer has switched to vexide, but
> there are still two teams who will use vexmason in the coming season.

## Features

- [x] Integrate into the native VEX VSCode extension and bundle files behind-
      the-scenes.
- [x] Read a config file and substitute defined constants.
- [ ] Completely replace the VEXCode VSCode extension for easier installation
      and better DX

## Installation

> [!NOTE]
>
> vexmason is only supported on Windows for now. The code is cross-platform,
> though, so as soon as I finish initial development I'll work on supporting
> Linux. Please let me know if you would like to try to build on OSX.
>
> The maintainer now uses a Linux laptop, so it's a bit ironic.

Installation is easy. Just head over to the
[GitHub releases page](https://github.com/zabackary/vexmason/releases/) and
download/run the `installer.exe` file associated with the latest release.
Windows will (probably) flag the file as "unsafe". If you trust me, you can
ignore it. If you don't, read the code and
[compile it from the source](#Development) yourself.

## Configuration

vexmason will only run on directories with both a `vex_project_settings.json`
file AND a `vexmason-config.json` file in the `.vscode` directory. You should
also have a `vexmason-local-config.json` file there too. The format is:

`vexmason-config.json`

```json
{
  "config_version": "1.1",
  "name": "{{ defines/__AUTONOMOUS_ROUTE__ }} | vexmason template",
  "description": "A description. If ommited, vexmason will generate one for you.",
  "language": "python",
  "default_defines": {
    "__AUTONOMOUS_ROUTE__": "route1"
  }
}
```

`vexmason-local-config.json`

```json
{
  "config_version": "1.1",
  "computer_name": "your computer name, can be used like {{ computer-name }} in `name` and `description` fields",
  "defines_overrides": {
    "__COMPETITION_MODE__": false
  }
}
```

Note `config_version`: it indicates the vexmason config version the config was
written for. vexmason will error if it doesn't support the version. The latest
version is `1.1` with support for types in defines.

## Development

### Building

You'll need the Rust compiler (rustc) and Cargo installed to build this project,
as it's written in [Rust](https://www.rust-lang.org/). The easiest way to do
this is through [`rustup`](https://rustup.rs/).

Once you've done that, you can build the project with `cargo build` and/or run
the code with `cargo run --bin {binary name}`. There are three main binaries
built: the `vexcom` binary, the main binary ("`vexmason`"), and the installer
("`installer`").

### Steps to put the binaries in the right place

1. Run the installer. That's it. It will fetch the binaries from GitHub, though,
   so make sure you trust me.

or

1. Navigate to
   `~/.vscode/extensions/vexcode-{version}/resources/tools/vexcom/{platform}`
   and rename the existing `vexcom` or `vexcom.exe` to `vexcom.old` or
   `vexcom.old.exe`. Tip: make a backup and name it something like `vexcom.bak`
   beforehand in case something goes wrong.
2. Take your built `vexcom.exe` file (the one you built in
   [Building](#Building)) and make a copy of it in that directory.
3. Create the installation directory at `%LOCALAPPDATA%/vexmason` and
   `git clone` [`python-compiler`](https://github.com/zabackary/python-compiler)
   into `./lib/python-compiler`.
4. Make a copy of your built `vexmason.exe` and put it in `./bin`.
