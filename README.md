libfuzzer-mutation-graph-tool
====


LLVM 12 [added](https://github.com/llvm/llvm-project/commit/1bb1eac6b177739429e78703b265e7546792fd64) `-mutation_graph_file` option to dump seed tree.
This option has the following function (cited from their help message).

> Saves a graph (in DOT format) to mutation_graph_file. The graph contains a vertex for each input that has unique coverage; directed edges are provided between parents and children where the child has unique coverage, and are recorded with the type of mutation that caused the child.

*libfuzzer-mutation-graph-tool* is (maybe) useful to interact with libfuzzer's mutation graph file.

**NOTE: This tool is unstable**


Functions
----
Subcommands provides following functions.

```
libfuzzer-mutation-graph-tool 1.0
Nao Tomori (@K_atc)
A Tool to interact with libfuzzer's mutation graph file.

USAGE:
    libfuzzer-mutation-graph-tool <FILE> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <FILE>    A mutation graph file.

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    parse    Just parse mutation graph file.
    plot     Plot mutation graph file and save as PNG, SVG.
             This command requires graphviz.
    pred     List predecessor of given node.
```


Requirements
----
* Cargo & Rust 
    * Nightly required
* (Optonal) Graphviz
    * To plot dot file


How to install
----
### Using `cargo install`
```shell
cargo install --git https://github.com/K-atc/libfuzzer-mutation-graph-tool.git
```


How to build
----
```shell
cargo build
```


How to run
----
### `pred`
```shell
$ cargo run -q test/sample/mutation_graph_file/graph1.dot pred 93d7302ce24b88e8f9c27e37871cc72502aff5e2
adc83b19e793491b1c6ea0fd8b46cd9f32e592fc
a2dfa9429bf2a04d8f23fe980209bd5315f80523
47ded72503d8ca82bbd9d2291fd1ea4ad6b1453c
```