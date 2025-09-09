# Overview of Validating Claim 1

**Claim 1**: The ratio of crashed nodes (i.e., *f_l/n*, where *f_l* is the number of crashed nodes and *n* is the number of nodes) will enhance the attack success rate (ASR) of all attacking strategies on both Tusk and Bullshark. Such improvements in Tusk is more than that in Bullshark.

The Claim 1 is responding to Figures 7 and 8 in the paper, where we set *n=10*, *f_a=5*, *f_w=2* (i.e., the setting same as the second sub-figure in both Figure 7 and Figure 8).

This claim will involve two branches in this repo: tusk and bullshark. In the `README` of each branch, we provide detailed instructions on how to set up the environment and run the experiments. We recommend following the steps provided in each branch to validate Claim 1, as it would help you identify the errors if something goes wrong. However, for your convenience, we provide the following scripts to run relevant experiments in two single commands.

## Run experiments

### Evaluate Tusk

First, we evaluate the attacks in Tusk. In a new terminal window, under this directory (i.e.,`./claims/claim1` of the main branch), execute:

```bash
bash run_tusk.sh
```

This command will switch to the tusk branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (crash node ratio: 0/10): 38.33%
Baseline (crash node ratio: 2/10): 39.42%
Fissure (crash node ratio: 0/10): 68.42%
Fissure (crash node ratio: 2/10): 79.31%
Sluggish (crash node ratio: 0/10): 45.83%
Sluggish (crash node ratio: 2/10): 44.14%
Speculative (crash node ratio: 0/10): 58.21%
Speculative (crash node ratio: 2/10): 70.83%
```

These will be the expected results for Claim 1 validation. In particular, the results show that all three attacking strategies (i.e., Fissure, Sluggish, Speculative) can have attack success rates improved compared to the Baseline with the ratio of crashed nodes increasing (i.e., from 0/10 to 2/10).

### Evaluate Bullshark

Then, we evaluate the attack in Bullshark. In a new terminal window, navigate to the same directory (i.e.,`./claims/claim1` of the main branch), execute:

```bash
bash run_bullshark.sh
```

This command will switch to the Bullshark branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (crash node ratio: 0/10): 42.64%
Baseline (crash node ratio: 2/10): 50.77%
Fissure (crash node ratio: 0/10): 72.6%
Fissure (crash node ratio: 2/10): 74.82%
Sluggish (crash node ratio: 0/10): 42.71%
Sluggish (crash node ratio: 2/10): 55.56%
Speculative (crash node ratio: 0/10): 61.54%
Speculative (crash node ratio: 2/10): 63.51%
```

These will be the expected results for Claim 1 validation. In particular, the results show that all three attacking strategies (i.e., Fissure, Sluggish, Speculative) can have attack success rates improved compared to the Baseline with the ratio of crashed nodes increasing (i.e., from 0/10 to 2/10). Moreover, we find that compared to the attacks in Tusk, the attacks in Bullshark benefit less from the ratio of crashed nodes.

## Result deviations

There might be several reasons that you might get results slightly different.

### 1. Machine specification

This experiment is very resouce-consuming, as it will deploy 10 nodes, 10 clients, and 20 workers. In our evaluation, we use an Ubuntu server equipped with 48 CPU cores and 128 GB of RAM, and it works well. If you are using a machine with weak spec, the results might be slightly different, since the exhausted resources could block the attacking process. We would suggest using a machine with >32 GB of RAM, or otherwise, try to run the experiment with fewer nodes (for example, by setting *n=4*, *f_l=1* in the `articrash` function of `./benchmark/fabfile.py` of the Tusk/Bullshark branch).

### 2. Repeat Evaluation

In the above experiments, each measurement only contains 0~30 times of attacks. This could save your time in getting a rough comparison (which already show the effectiveness of the attacks), but might have deviations. Moreover, in our attacking evaluation, we randomly generate blocks to simulate the attacking progress. There could be some cases that no blocks can be attacked, and the results could be missing. If you find such cases happen or wish to get a more precise result, you could repeat this progress multiple times, and the results are cumulative.

> Note: In our paper, each measure contains at least 500 times of attacks.
