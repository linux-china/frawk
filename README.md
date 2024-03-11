zawk
===============

zawk is a small programming language for writing short programs processing textual data. 
To a first approximation, it is an implementation of the [AWK](https://en.wikipedia.org/wiki/AWK) language; 
many common Awk programs produce equivalent output when passed to frawk. 
You might be interested in zawk if you want your scripts to handle escaped CSV/TSV like standard Awk fields,
or if you want your scripts to execute faster,
or if you want a standard AWK library to make life easy.

The info subdirectory has more in-depth information on zawk:

* [Overview](info/overview.md):
  what frawk is all about, how it differs from Awk.
* [Types](info/types.md): A
  quick gloss on frawk's approach to types and type inference.
* [Parallelism](info/parallelism.md):
  An overview of frawk's parallelism support.
* [Benchmarks](info/performance.md):
  A sense of the relative performance of frawk and other tools when processing
  large CSV or TSV files.
* [Standard Library](info/stdlib.md):
  A standard library by zawk, including exciting functions that are new when compared with Awk.

frawk is dual-licensed under MIT or Apache 2.0.

## Installation

*Note: frawk uses some nightly-only Rust features by default.
Build [without the `unstable`](#building-using-stable)
feature to build on stable.*  

You will need to [install Rust](https://rustup.rs/). If you have not updated rust in a while, 
run `rustup update nightly` (or `rustup update` if building using stable). If you would like
to use the LLVM backend, you will need an installation of LLVM 12 on your machine: 

* See [this site](https://apt.llvm.org/) for installation instructions on some debian-based Linux distros.
  See also the comments on [this issue](https://github.com/ezrosent/frawk/issues/63) for docker files that
  can be used to build a binary on Ubuntu.
* On Arch `pacman -Sy llvm llvm-libs` and a C compiler (e.g. `clang`) are sufficient as of September 2020.
* `brew install llvm@15` or similar seem to work on Mac OS.

Depending on where your package manager puts these libraries, you may need to
point `LLVM_SYS_150_PREFIX` at the llvm library installation (e.g.
`/usr/lib/llvm-15` on Linux or `/opt/homebrew/opt/llvm@15` on Mac OS when installing llvm@15 via Homebrew).

### Building Without LLVM

While the LLVM backend is recommended, it is possible to build frawk only with
support for the Cranelift-based JIT and its bytecode interpreter. To do this,
build without the `llvm_backend` feature. The Cranelift backend provides
comparable performance to LLVM for smaller scripts, but LLVM's optimizations
can sometimes deliver a substantial performance boost over Cranelift (see the
[benchmarks](info/performance.md)
document for some examples of this).

### Building Using Stable

frawk currently requires a nightly compiler by default. To compile frawk using stable,
compile without the `unstable` feature. Using `rustup default nightly`, or some other
method to run a nightly compiler release is otherwise required to build frawk.

### Building a Binary

With those prerequisites, cloning this repository and a `cargo build --release`
or `cargo [+nightly] install --path <frawk repo path>` will produce a binary that you can
add to your `PATH` if you so choose:

```
$ cd <frawk repo path>
# With LLVM
$ cargo +nightly install --path .
# Without LLVM, but with other recommended defaults
$ cargo +nightly install --path . --no-default-features --features use_jemalloc,allow_avx2,unstable
```

frawk is now on [crates.io](https://crates.io/crates/zawk), so running 
`cargo +nightly install zawk` with the desired features should also work.


## Bugs and Feature Requests

frawk has bugs, and many rough edges. If you notice a bug in frawk, filing an issue
with an explanation of how to reproduce the error would be very helpful. There are
no guarantees on response time or latency for a fix. No one works on frawk full-time.
The same policy holds for feature requests.

## Credits

Thanks to Eli Rosenthal's [frawk](https://github.com/ezrosent/frawk).
zawk is based on frawk. Without frawk, there would be no zawk. 