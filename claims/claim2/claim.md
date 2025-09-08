# Overview of Validating Claim 2

**Claim 2**: The speculative attack benefits from the number of workers (i.e., *f_w*) mostly on both Tusk and Bullshark.

The Claim 2 is responding to Figure 9 in the paper, where we set *n=10*, *f_a=3*, *f_l=0* (i.e., the setting same as the first and third sub-figures in both Figure 9).

This claim will involve two branches in this repo: tusk and bullshark. In the `README` of each branch, we provide detailed instructions on how to set up the environment and run the experiments. We recommend following the steps provided in each branch to validate Claim 2, as it would help you identify the errors if something goes wrong. However, for your convenience, we provide the following scripts to run relevant experiments in two single commands.

## Run experiments

### Evaluate Tusk

First, we evaluate the attacks in Tusk. In a new terminal window, under this directory, execute:

```bash
bash run_tusk.sh
```

This command will switch to the tusk branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (workers: 2): 43.4%
Baseline (workers: 8): 29.17%
Fissure (workers: 2): 66.67%
Fissure (workers: 8): 66.67%
Sluggish (workers: 2): 55.0%
Sluggish (workers: 8): 54.17%
Speculative (workers: 2): 51.52%
Speculative (workers: 8): 72.6%
```

These will be the expected results for Claim 2 validation. In particular, the results show that the Speculative attack can benefit from the number of workers (i.e., from *f_w=2* to *f_w=8*) mostly in Tusk.

### Evaluate Bullshark

Then, we evaluate the attack in Bullshark. In a new terminal window, under this directory, execute:

```bash
bash run_bullshark.sh
```

This command will switch to the Bullshark branch, install dependencies, compile the codes, and finally run the experiments. It might take you 50 minutes if you haven't installed relevant dependencies before. If you have all required dependencies, the experiment itself would take you around 32 minutes. After it terminates, the terminal will print the following information:

```bash
...
Baseline (workers: 2): 47.52%
Baseline (workers: 8): 55.77%
Fissure (workers: 2): 40.62%
Fissure (workers: 8): 60.61%
Sluggish (workers: 2): 46.03%
Sluggish (workers: 8): 40.91%
Speculative (workers: 2): 53.37%
Speculative (workers: 8): 85.56%

```

These will be the expected results for Claim 2 validation. In particular, the results show that the Speculative attack can benefit from the number of workers (i.e., from *f_w=2* to *f_w=8*) mostly in Bullshark.

## Result deviations

There might be several reasons that you might get results slightly different.

### 1. Machine specification

This experiment is very resouce-consuming, as it will deploy 10 nodes, 10 clients, and 20~80 workers. In our evaluation, we use an Ubuntu server equipped with 48 CPU cores and 128 GB of RAM, and it works well. If you are using a machine with weak spec, the results might be slightly different, since the exhausted resources could block the attacking process. We would suggest using a machine with >32 GB of RAM, or otherwise, try to run the experiment with fewer nodes (for example, by setting *n=4*, *f_w=4* in the `artiworker` function of `./benchmark/fabfile.py` of the Tusk/Bullshark branch).

### 2. Repeat Evaluation

In the above experiments, each measurement only contains 0~30 times of attacks. This could save your time in getting a rough comparison (which already show the effectiveness of the attacks), but might have deviations. Moreover, in our attacking evaluation, we randomly generate blocks to simulate the attacking progress. There could be some cases that no blocks can be attacked, and the results could be missing. If you find such cases happen or wish to get a more precise result, you could repeat this progress multiple times, and the results are cumulative.

> Note: In our paper, each measure contains at least 500 times of attacks.
