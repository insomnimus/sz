# rs-size

Tiny tool that calculates file sizes, written in rust.

# Features

-	Very fast.
-	Display in human readable or plain byte.
-	Supports glob patterns on windows (linux already has glob support via bash and most other shells).

# Installation

You'll need cargo and the rust compiler, preferably v1.5 and above.
Note that, after installing, the binary will be named `rsize`.

You have two options to install rs-size:

### Installation Using Git

```
git clone https://www.github.com/insomnimus/rs-size
cd rs-size
cargo install --path .
```

### Installation Using Cargo

`cargo install --git https://www.github.com/insomnimus/rs-size.git`

# Usage

If you followed any of the installation steps above, you should have the rs-size binary (installs as rsize) in your cargo home path (or ~/.cargo/bin).

The usage is very simple, you just give rsize some file names and it coutns the sizes.
If called without args, it calculates the size of the current directory.
Call with `-b` to get the byte count without pretty printing.
