# Intended use

This repo is for the Artifact Evaluation of ACSAC 2025 proceeding: "[No Fish Is Too Big for Flash Boys! Frontrunning on DAG-based Blockchains](https://eprint.iacr.org/2024/1496)". It provides the main code and instructions for readers to better understand how a new frontrunning attack can be conducted in DAG-based protocols and to reproduce the results claimed in the paper. It is designed for academic study but not for commercial usage.

# Limitations

The code cannot be used to conduct inter-block frontrunning attacks on real-world systems. Our evaluation contains a random generation of victim blocks, which does not reflect the real-world cases. Moreover, the deployment of nodes in our repo is different from the node distributions in the real world, which introduces latency differences that might affect the attack success rate in practice.
