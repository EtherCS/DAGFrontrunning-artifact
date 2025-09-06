# No Fish Is Too Big for Flash Boys! Frontrunning on DAG-based Blockchains - ACSAC 2025 Artifact
This is the repository for the Artifact Evaluation of ACSAC 2025  proceeding: "[No Fish Is Too Big for Flash Boys! Frontrunning on DAG-based Blockchains](https://eprint.iacr.org/2024/1496)".

This branch is the attack evaluation on Tusk.

## Quick Start

The core protocols are written in Rust, but all benchmarking scripts are written in Python and run with [Fabric](http://www.fabfile.org/).
To deploy and benchmark a testbed of 10 nodes on your local machine, download the repo and install the python dependencies:

```
$ down the anonymous project
$ cd [this repo name]/benchmark
$ pip install -r requirements.txt
```

You also need to install Clang (required by rocksdb) and [tmux](https://linuxize.com/post/getting-started-with-tmux/#installing-tmux) (which runs all nodes and clients in the background). Finally, run a local benchmark using fabric:

```
$ fab local
```

This command may take a long time the first time you run it (compiling rust code in `release` mode may be slow) and you can customize a number of benchmark parameters in the `local` function in `fabfile.py`. The defaults parameters are to evaluate the pick-maximum attack one time with 10 nodes (3 nodes are arbitragers) and 4 workers for each node. When the benchmark terminates, it creates a file `local-3-3-0-4-10.txt` in `.\results\tusk` with the below output.

```
-----------------------------------------
 SUMMARY:
-----------------------------------------
 + CONFIG:
 Attacking type: 3 
 Arbitragers: 3 node(s)
 Faults: 0 node(s)
 Committee size: 10 node(s)
 Worker(s) per node: 4 worker(s)
 Collocate primary and workers: True
 Input rate: 30,000 tx/s
 Transaction size: 512 B
 Execution time: 120 s

 Header size: 1,000 B
 Max header delay: 200 ms
 GC depth: 50 round(s)
 Sync retry delay: 10,000 ms
 Sync retry nodes: 3 node(s)
 batch size: 500,000 B
 Max batch delay: 200 ms

 + RESULTS:
 Consensus TPS: 25,976 tx/s
 Consensus BPS: 13,299,642 B/s
 Consensus latency: 983 ms

 End-to-end TPS: 25,960 tx/s
 End-to-end BPS: 13,291,300 B/s
 End-to-end latency: 1,286 ms

 + ATTACK:
 Speculative front-running rate: 71.43% (15/21) 
-----------------------------------------

Cumulative speculative front-running results: 71.43% (15/21) 
```


## Evaluation on AWS

Please refer to the tutorial provided by [Narwhal AWS Evaluation](https://github.com/asonnino/narwhal/tree/master/benchmark), which explains how to benchmark the codebase and read benchmarks' results. It also provides a step-by-step tutorial to run benchmarks on [Amazon Web Services (AWS)](https://aws.amazon.com) accross multiple data centers (WAN).
