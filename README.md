seed-tree-analyzer
====


LLVM 12 [added](https://github.com/llvm/llvm-project/commit/1bb1eac6b177739429e78703b265e7546792fd64) `-mutation_graph_file` option to dump seed tree.
This option has the following function (cited from their help message).

> Saves a graph (in DOT format) to mutation_graph_file. The graph contains a vertex for each input that has unique coverage; directed edges are provided between parents and children where the child has unique coverage, and are recorded with the type of mutation that caused the child.

*seed-tree-analyzer* is (maybe) useful to interact with libfuzzer's mutation graph file and AFL's seed tree.

**NOTE: This tool is unstable**


Functions
----
### For libfuzzer
Subcommands provides following functions.

```
$ cargo run -q --bin seed-tree-analyzer-libfuzzer --features libfuzzer -- help
seed-tree-analyzer-libfuzzer 1.0
Nao Tomori (@K_atc)
A Tool to interact with libfuzzer's mutation graph file.

USAGE:
    seed-tree-analyzer-libfuzzer <FILE> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <FILE>    A mutation graph file.

SUBCOMMANDS:
    deriv     Analyze derivation of OFFSET of NODE_NAME
    help      Prints this message or the help of the given subcommand(s)
    leaves    List leaf nodes.
    ls        List nodes.
    origin    Find origin seeds on each offset of NODE_NAME
    parse     Just parse mutation graph file.
    plot      Plot mutation graph file and save as PNG, SVG.
              This command requires graphviz.
    pred      List predecessor of given node.
    roots     List root nodes.
```

### For AFL
```
$ cargo run -q --bin seed-tree-analyzer-afl --features afl -- help
seed-tree-analyzer-afl 1.0
Nao Tomori (@K_atc)
A Tool to interact with AFL's seed tree described in inputs file name.

USAGE:
    seed-tree-analyzer-afl <INPUT_DIR>... [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <INPUT_DIR>...    Directories contains AFL's input files.

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    parse    Scan INPUT_DIR(s) and output seed tree in dot format.
    plot     Plot and save seed tree as DOT, PNG, SVG.
             This command requires graphviz.
```


Requirements
----
* Cargo & Rust 
    * Nightly required
* (Optional) Graphviz
    * To render dot file


How to install
----
### Using `cargo install`
```shell
cargo install --git https://github.com/K-atc/seed-tree-analyzer.git --bins --all-features
```

Or manually git clone and:

```shell
cargo install --path . --bins --all-features
```


How to build
----
```shell
cargo build --bins --all-features
```


How to run `seed-tree-analyzer-libfuzzer`
----
### `pred`
List predecessors of `93d730`.

```shell
$ seed-tree-analyzer-libfuzzer test/sample/mutation_graph_file/graph1.dot pred 93d7302ce24b88e8f9c27e37871cc72502aff5e2
adc83b19e793491b1c6ea0fd8b46cd9f32e592fc
a2dfa9429bf2a04d8f23fe980209bd5315f80523
47ded72503d8ca82bbd9d2291fd1ea4ad6b1453c
```

### Diffing predecessor of crash input
Assume we got crash input based on `c298122410da09836c59484e995c287294c31394`.
The following is an output of libfuzzer:

```
==10928==ABORTING
MS: 1 ChangeBinInt-; base unit: c298122410da09836c59484e995c287294c31394
```

We can observe how seeds which are predecessors of `c29812` were generated, using *libfuzzer-mutation-graph-tool*:

```
$ seed-tree-analyzer-libfuzzer test/sample/mutation_graph_file/fuzzer-test-suite-openssl-1.0.1f.dot pred c298122410da09836c59484e995c287294c31394 --diff test/sample/seeds/fuzzer-test-suite-openssl-1.0.1f/
adc83b19e793491b1c6ea0fd8b46cd9f32e592fc -> c5c050e132b1ee3a4f627b3b0350b77737f5f181
        Insert (offset=0x0, bytes=[2b])
        Insert (offset=0x1, bytes=[0e])
c5c050e132b1ee3a4f627b3b0350b77737f5f181 -> 9609c0ae86c0bf1115d2c04655269e4f9271ef1f
        Replace(offset=0x0, length=0x3, bytes=[2e 03 18 2e 03 18])
9609c0ae86c0bf1115d2c04655269e4f9271ef1f -> c7d46cfc565b9ca12c066cd242b27a38815d9b9f
        Delete (offset=0x3, length=0x1)
c7d46cfc565b9ca12c066cd242b27a38815d9b9f -> a017eb80d559e0b3a84b68c802b9adc51aa54cc7
        Replace(offset=0x2, length=0x1, bytes=[00 00])
        Replace(offset=0x4, length=0x1, bytes=[fe e3 e3 2e 03 00 00 00 10 03 00 00 00 00 00 b7 00 30])
a017eb80d559e0b3a84b68c802b9adc51aa54cc7 -> c396417d7c899b5498a4893c11e63b227706911e
        Replace(offset=0xd, length=0x1, bytes=[2e])
        Insert (offset=0x11, bytes=[03 fe e3 e3 2e 03])
c396417d7c899b5498a4893c11e63b227706911e -> 99878cf124782dc6d21f079bb29e0dba54606bbb
        Insert (offset=0x1b, bytes=[03 00 00 03 fe e3 e3 2e 03 00 00 03 fd b7])
        Insert (offset=0x1d, bytes=[03 00 00 03 fe e3 00 30])
99878cf124782dc6d21f079bb29e0dba54606bbb -> d17b6ed1c3a693b75da5b4b57976296c8ea01169
        Delete (offset=0x6, length=0x2)
        Replace(offset=0xa, length=0x4, bytes=[18 03 18 00 00 2e])
        Replace(offset=0x2d, length=0x2, bytes=[bf])
        Replace(offset=0x30, length=0x3, bytes=[ff ff ff ff 2e 03 ff])
d17b6ed1c3a693b75da5b4b57976296c8ea01169 -> 573a46286deaf9df81fb90d7b786708d845b5f23
        Replace(offset=0x5, length=0x2, bytes=[02 da])
        Replace(offset=0xd, length=0x1, bytes=[16])
        Replace(offset=0x11, length=0x1, bytes=[0b])
        Delete (offset=0x14, length=0x6)
        Replace(offset=0x1b, length=0x1, bytes=[fd])
        Delete (offset=0x1d, length=0x1)
        Replace(offset=0x1f, length=0x4, bytes=[00])
        Insert (offset=0x24, bytes=[02 da])
        Delete (offset=0x25, length=0x11)
573a46286deaf9df81fb90d7b786708d845b5f23 -> dd0d17f2261fa314c23cd3ab442f3e4b1279e5ca
        Replace(offset=0xd, length=0x1, bytes=[18])
        Replace(offset=0x12, length=0x2, bytes=[01 10])
dd0d17f2261fa314c23cd3ab442f3e4b1279e5ca -> 76e46ec1efcdcb854486037defc3e777a62524ed
        Replace(offset=0x13, length=0x3, bytes=[00 03 fe])
76e46ec1efcdcb854486037defc3e777a62524ed -> c298122410da09836c59484e995c287294c31394
        Replace(offset=0x1a, length=0x1, bytes=[1d])
```


How to run `seed-tree-analyzer-afl`
----
TODO

### List predecessors of a node with given file hash
```
$ seed-tree-analyzer-afl ./findings/ preds --hash 3a54435a70b7390ac4edfba3c274f36f9afb8d61 --meta
MutationGraphNode { name: "not_kitty_icc.png", crashed: false, file: "", hash: "" }
MutationGraphNode { name: "000003", crashed: false, file: "./findings/queue/id:000003,orig:not_kitty_icc.png", hash: "641af7f3bf9a7531b34fb27b6ce3de791c6a8f60" }
MutationGraphNode { name: "000221", crashed: false, file: "./findings/queue/id:000221,src:000003,op:flip1,pos:34,+cov", hash: "00156d140a3f0f6de68057f46337610c275f307e" }
MutationGraphNode { name: "001092", crashed: false, file: "./findings/queue/id:001092,src:000221+000780,op:splice,rep:128,+cov", hash: "5145f15bbdf4cb45eeffa61d89ae48b92a1ee475" }
MutationGraphNode { name: "001094", crashed: false, file: "./findings/queue/id:001094,src:001092,op:arith8,pos:37,val:+17,+cov", hash: "1b7cbf8dc43f380d8091d6c74ba05dbe5316716a" }
MutationGraphNode { name: "001096", crashed: false, file: "./findings/queue/id:001096,src:001094,op:int32,pos:33,val:+0,+cov", hash: "3a0f83cd8c2c51c5fc837111cf32ae047a518068" }
MutationGraphNode { name: "001239", crashed: false, file: "./findings/queue/id:001239,src:001096+000171,op:splice,rep:16,+cov", hash: "70a34b49b7a637642868e4b0b9a646efd70f95eb" }
MutationGraphNode { name: "crash-000066", crashed: true, file: "./findings/crashes/id:000066,sig:11,src:001239+000892,op:splice,rep:32", hash: "3a54435a70b7390ac4edfba3c274f36f9afb8d61" }
```