# Overview of Validating Claim 3

**Claim 3**: The fissure attack benefits from the ratio of attackers (i.e., *f_a/n*) mostly on both Tusk and Bullshark.

The Claim 3 is responding to Figures 13 and 14 in the paper, where we set *n=10*, *f_w=2*, *f_l=0* (i.e., the setting same as the first sub-figure in both Figure 13 and Figure 14).

This claim will involve two branches in this repo: tusk and bullshark. In the `README` of each branch, we provide detailed instructions on how to set up the environment and run the experiments. We recommend following the steps provided in each branch to validate Claim 3, as it would help you identify the errors if something goes wrong. However, for your convenience, we provide the following scripts to run relevant experiments in two single commands.

## Run experiments

### Evaluate Tusk

First, we evaluate the attacks in Tusk. In a new terminal window, under this directory, execute:

```bash
bash run_tusk.sh
```

This command will switch to the tusk branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (frontrunners: 3): 39.71%
Baseline (frontrunners: 5): 36.49%
Fissure (frontrunners: 3): 42.86%
Fissure (frontrunners: 5): 57.83%
Sluggish (frontrunners: 3): 40.0%
Sluggish (frontrunners: 5): 38.46%
Speculative (frontrunners: 3): 49.32%
Speculative (frontrunners: 5): 55.45%

```

These will be the expected results for Claim 3 validation. In particular, the results show that the fissure attack benefits from the ratio of attackers (i.e., from *f_a/n=* 3/10 to 5/10) mostly in Tusk.

### Evaluate Bullshark

Then, we evaluate the attack in Bullshark. In a new terminal window, under this directory, execute:

```bash
bash run_bullshark.sh
```

This command will switch to the Bullshark branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (frontrunners: 3): 46.67%
Baseline (frontrunners: 5): 45.35%
Fissure (frontrunners: 3): 41.67%
Fissure (frontrunners: 5): 72.73%
Sluggish (frontrunners: 3): 43.86%
Sluggish (frontrunners: 5): 43.51%
Speculative (frontrunners: 3): 50.35%
Speculative (frontrunners: 5): 62.79%
```

These will be the expected results for Claim 3 validation. In particular, the results show that the fissure attack benefits from the ratio of attackers (i.e., from *f_a/n=* 3/10 to 5/10) mostly in Bullshark.

## Result deviations

There might be several reasons that you might get results slightly different.

### 1. Machine specification

This experiment is very resouce-consuming, as it will deploy 10 nodes, 10 clients, and 20 workers. In our evaluation, we use an Ubuntu server equipped with 48 CPU cores and 128 GB of RAM, and it works well. If you are using a machine with weak spec, the results might be slightly different, since the exhausted resources could block the attacking process. We would suggest using a machine with >32 GB of RAM, or otherwise, try to run the experiment with fewer nodes (for example, by setting *n=4*, *f_a=1, 3* in the `artifrontrunner` function of `./benchmark/fabfile.py` of the Tusk/Bullshark branch).

### 2. Repeat Evaluation

In the above experiments, each measurement only contains 0~30 times of attacks. This could save your time in getting a rough comparison (which already show the effectiveness of the attacks), but might have deviations. Moreover, in our attacking evaluation, we randomly generate blocks to simulate the attacking progress. There could be some cases that no blocks can be attacked, and the results could be missing. If you find such cases happen or wish to get a more precise result, you could repeat this progress multiple times, and the results are cumulative.

> Note: In our paper, each measure contains at least 500 times of attacks.
