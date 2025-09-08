# No Fish Is Too Big for Flash Boys! Frontrunning on DAG-based Blockchains - ACSAC 2025 Artifact

This is the repository for the Artifact Evaluation of ACSAC 2025 proceeding: "[No Fish Is Too Big for Flash Boys! Frontrunning on DAG-based Blockchains](https://eprint.iacr.org/2024/1496)". In this paper, we define a new type of attack called inter-block frontrunning attack towards Directed Acyclic Graph (DAG)-based consensus protocols. In brief, the inter-block frontrunning attack aims to manipulate the order of blocks using some dedicated attacking strategies, by which the attacker can make its transactions ordered before the target/victim transactions. This repo provides the implementation of all attacking strategies proposed in the paper: (i) Fissure attack; (ii) Speculative attack; (iii) Sluggish attack.

## Table of Contents

1. [Artifact Overview](#artifact-overview)
2. [Evaluation Claims](#evaluation-claims)
3. [Install Dependencies](#install-dependencies)
4. [Run Experiments](#run-experiments)

## Artifact Overview

This artifact contains the implementation of all three proposed attacking strategies in the paper, and automation scripts used to reproduce experiment results.

Our implementation is built on two open source DAG-based consensus protocols: [Tusk](https://github.com/asonnino/narwhal/tree/master) and [Bullshark](https://github.com/asonnino/narwhal/tree/bullshark), both of which are implemented in Rust, use the asynchronous runtime Tokio, ed25519-dalek for signatures, and RocksDB as the underlying database. Tusk and Bullshark have the same code structure except for their consensus logic in ```./consensus/src/lib.rs```. In this repo, the [Tusk](https://github.com/EtherCS/DAGFrontrunning-artifact/tree/tusk) branch provides the implementation of frontrunning evaluation on Tusk, while the [bullshark](https://github.com/EtherCS/DAGFrontrunning-artifact/tree/bullshark) branch is for the frontrunning evaluation on Bullshark. The attacking logic are mainly located in ```./primary/src/core.rs``` (which mainly specifies how to handle different messages) and ```./primary/src/proposer.rs``` (which specifies how a node creates a new block).

### Machine specification

In our evaluation, we tried two different machines, and both of them work. These include: (1) a server running Ubuntu 22.04 and is equipped with 48 CPU cores, 128 GB of RAM, and 10 TB SSD; (2) a MackBook Pro 2020 with 8 CPU cores, 16 GB of RAM, and 500 GB SSD.

## Evaluation Claims

This paper presents a new type of attack to manipulate the block/transaction order in two representative DAG-based consensus protocols---Tusk and Bullshark. We developed three concrete attacking strategies to increase the attack success rate (ASR), including Fissure attacking strategy, Speculative attacking strategy, and Sluggish attacking strategy. To show the effectiveness of the attacking strategies, we choose the baseline, where all nodes perform correctly (i.e., no attacking behaviors are performed). We compare the ASRs achieved by different attacking strategies and that achieved by the baseline under different configurations and settings, including the number of nodes *n*, the number of attackers *f_a*, the number of crashed nodes *f_l*, and the number of workers *f_w*. The evaluation is to show different configurations/factors can enhance the ASRs of different attacking strategies.

- **Main claim 1**: The ratio of crashed nodes (i.e., *f_l/n*, where *f_l* is the number of crashed nodes and *n* is the number of nodes) will enhance the ASRs of all attacking strategies on both Tusk and Bullshark. Such improvements in Tusk is more than that in Bullshark.
- **Main claim 2**: The speculative attack benefits from the number of workers (i.e., *f_w*) mostly on both Tusk and Bullshark.
- **Main claim 3**: The fissure attack benefits from the ratio of attackers (i.e., *f_a/n*) mostly on both Tusk and Bullshark.

> Note: All the above evaluation claims can be validated by running the codes in a local machine. For more details of validating the claims, please refer to `./claims` folder to see step-by-step instructions.

## Install Dependencies

We will use the bullshark branch as an example to show how to install dependencies and run experiments throughout. Thus, before running the codes, switch to the bullshark branch by executing:
```
git checkout bullshark
git pull origin bullshark
```
> Note: The commands used to run experiment in the tusk branch are the same as that used in the bullshark branch. But remember to switch to the tusk branch first.

### Install dependencies

#### Software dependencies

This project requires the following software dependencies:
- python3, pip3
- Rust (we used 1.86.0 during the evaluation)
- Clang (required by Rocksdb)
- [tmux](https://linuxize.com/post/getting-started-with-tmux/#installing-tmux) (which runs all nodes and clients in the background)

For convenience, we provide an installation script ```./install.sh``` in the main branch. Run the following command to automatically install all software dependencies:
```bash
bash ./install.sh
```

#### Python dependencies

The core protocols are written in Rust, but all benchmarking scripts are written in Python and run with [Fabric](http://www.fabfile.org/). Thus, we need to install the Python dependencies that are specified in ```./benchmark/requirements.txt```. Briefly, after the software installation completes, run the commands:
```bash
cd benchmark
pip3 install -r requirements.txt
```

## Run Experiments

### Build binaries and quick start

The python script in ```./benchmark/fabfile.py``` provides interfaces to run experiments. Specifically, under the ```./benchmark``` directory, execute the following command to run a local benchmark using fabric:

```bash
fab local
```

This command may take a long time the first time you run it (compiling rust code in `release` mode may be slow) and you can customize a number of benchmark parameters in the `local` function in `fabfile.py`. The defaults parameters are to evaluate the speculative attack one time with 10 nodes (3 nodes are frontrunning attackers) and 4 workers for each node. When the benchmark terminates, it creates a file `local-3-3-0-4-10.txt` in `.\results\bullshark` with the below output.

```bash
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
 Execution time: 30 s

 Header size: 1,000 B
 Max header delay: 200 ms
 GC depth: 50 round(s)
 Sync retry delay: 10,000 ms
 Sync retry nodes: 3 node(s)
 batch size: 500,000 B
 Max batch delay: 200 ms

 + RESULTS:
 Consensus TPS: 29,308 tx/s
 Consensus BPS: 15,005,584 B/s
 Consensus latency: 503 ms

 End-to-end TPS: 29,129 tx/s
 End-to-end BPS: 14,914,022 B/s
 End-to-end latency: 752 ms

 + ATTACK:
 Speculative front-running rate: 77.78% (7/9) 
-----------------------------------------

Cumulative speculative front-running results: 77.78% (7/9) 
```

### Run experiments with customized parameters

For more details about running experiments with different configurations and interpreting the results, please refer to ReadMe of the [bullshark](https://github.com/EtherCS/DAGFrontrunning-artifact/tree/bullshark) branch.
