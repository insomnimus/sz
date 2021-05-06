# sz

Tiny tool that calculates file sizes, written in rust.

# Features

-	Very fast.
-	Display in human readable or plain byte.
-	Supports glob patterns on windows (linux already has glob support via bash and most other shells).

# Installation

You'll need cargo and the rust compiler, preferably v1.50 and above.

You have two options to install sz:

### Installation Using Git

```
git clone https://github.com/insomnimus/sz
cd sz
git checkout main
cargo install --path .
```

### Installation Using Cargo

`cargo install --git https://github.com/insomnimus/sz --branch main`

# Usage

If you followed any of the installation steps above, you should have the sz binary in your cargo home path (or ~/.cargo/bin).

The usage is very simple, you just give sz some file names and it calculates the sizes.
If called without args, it calculates the size of the current directory.
Call with `-b` to get the byte count without pretty printing.
