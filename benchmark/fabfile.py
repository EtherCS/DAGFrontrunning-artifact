# Copyright(C) Facebook, Inc. and its affiliates.
from fabric import task

from benchmark.local import LocalBench
from benchmark.logs import ParseError, LogParser
from benchmark.commands import CommandMaker
import subprocess

# from benchmark.utils import Print
from benchmark.utils import BenchError, Print, PathMaker, progress_bar
from benchmark.plot import Ploter, PlotError
from benchmark.instance import InstanceManager
from benchmark.remote import Bench, BenchError
from re import findall, search
from multiprocessing import Pool
from time import sleep
import time


@task
def local(ctx, debug=True):
    # local-{attack_type}-{arbitragers}-{faults}-{workers}-{nodes}.txt
    start_time = time.time()
    for at, arbis, ff, wks, nds, rs in [
        (3, 3, 0, 4, 10, 1),
    ]:

        """Run benchmarks on localhost"""
        runs = rs
        attack_type = at
        arbitragers = arbis
        faults = ff
        workers = wks
        nodes = nds
        bench_params = {
            # faults: the number of liveness attackers, i.e., t_l defined in our paper
            # self.json['authorities'][-faults:] primary nodes will be liveness attackers
            "faults": faults,
            # arbitragers: the number of arbitragers, i.e., t_a defined in our paper
            # self.json['authorities'][-(arbitragers+faults): -faults] primary nodes will be arbitragers
            "arbitragers": arbitragers,
            # 0: no attack; 1: fissure; 2: sluggish; 3: speculative; 10: monitor but do nothing (for comparison)
            "attack_type": attack_type,
            "nodes": nodes,
            "workers": workers,  # for speculative: more workers can create more blocks, workers*max_header_delay/max_batch_delay = the number of blocks the speculative attacker try
            "rate": 30_000,
            "tx_size": 512,
            "duration": 120,
        }
        node_params = {
            "header_size": 1_000,  # bytes
            "max_header_delay": 200,  # ms
            "gc_depth": 50,  # rounds, i.e., \mathcal{r}_r defined in our paper
            "sync_retry_delay": 10_000,  # ms
            "sync_retry_nodes": 3,  # number of nodes
            "batch_size": 500_000,  # bytes
            "max_batch_delay": 200,  # ms
        }
        try:
            filename = PathMaker.local_result_file(
                attack_type,
                arbitragers,
                faults,
                workers,
                nodes,
            )
            for i in range(runs):
                print(f"Local repeat {i}th\n")
                print(
                    f"type {attack_type}, arbs {arbitragers}, fault {faults}, workers {workers}, nodes {nodes}\n"
                )
                ret = LocalBench(bench_params, node_params).run(debug)
                # local_result_file(attack_type, arbitragers, faults, nodes)
                ret.print(filename)
                cmd = CommandMaker.kill().split()
                subprocess.run(cmd, stderr=subprocess.DEVNULL)
                sleep(5)
            if attack_type == 1:
                succ_num, total_num = _get_fissure_total(filename)
                fissure_total_result = ""
                if total_num != 0:
                    fissure_total_result = f"\nCumulative fissure front-running results: {round(succ_num/total_num*100, 2):,}% ({succ_num:,}/{total_num:,}) \n"
                with open(filename, "a") as f:
                    f.write(fissure_total_result)
            elif attack_type == 2:
                succ_num, total_num = _get_sluggish_attack_total(filename)
                sluggish_total_result = ""
                if total_num != 0:
                    sluggish_total_result = f"\nCumulative sluggish front-running results: {round(succ_num/total_num*100, 2):,}% ({succ_num:,}/{total_num:,}) \n"
                with open(filename, "a") as f:
                    f.write(sluggish_total_result)
            elif attack_type == 3:
                succ_num, total_num = _get_speculative_attack_total(filename)
                speculative_total_result = ""
                if total_num != 0:
                    speculative_total_result = f"\nCumulative speculative front-running results: {round(succ_num/total_num*100, 2):,}% ({succ_num:,}/{total_num:,}) \n"
                with open(filename, "a") as f:
                    f.write(speculative_total_result)
            elif attack_type == 10:
                succ_num, total_num = _get_monitor_total(filename)
                monitor_total_result = ""
                if total_num != 0:
                    monitor_total_result = f"\nCumulative non-strategy front-running results: {round(succ_num/total_num*100, 2):,}% ({succ_num:,}/{total_num:,}) \n"
                with open(filename, "a") as f:
                    f.write(monitor_total_result)

        except BenchError as e:
            Print.error(e)
    end_time = time.time()
    runtime_min = (end_time - start_time) / 60
    print("Runtime: ", runtime_min, "minutes")


def _get_monitor_total(path):
    log = ""
    with open(path, "r") as f:
        log += f.read()
    tmp = findall(r" Non-strategy front-running rate: \d+.\d+% \((\d+)\/(\d+)\) ", log)
    tmp = [(s, t) for s, t in tmp]
    succ_num = 0
    total_num = 0
    for s, t in tmp:
        succ_num += int(s)
        total_num += int(t)
    return succ_num, total_num


def _get_fissure_total(path):
    log = ""
    with open(path, "r") as f:
        log += f.read()
    tmp = findall(r" Fissure front-running rate: \d+.\d+% \((\d+)\/(\d+)\) ", log)
    tmp = [(s, t) for s, t in tmp]
    succ_num = 0
    total_num = 0
    for s, t in tmp:
        succ_num += int(s)
        total_num += int(t)
    return succ_num, total_num


def _get_sluggish_attack_total(path):
    log = ""
    with open(path, "r") as f:
        log += f.read()
    tmp = findall(r" Sluggish front-running rate: \d+.\d+% \((\d+)\/(\d+)\) ", log)
    tmp = [(s, t) for s, t in tmp]
    succ_num = 0
    total_num = 0
    for s, t in tmp:
        succ_num += int(s)
        total_num += int(t)
    return succ_num, total_num


def _get_speculative_attack_total(path):
    log = ""
    with open(path, "r") as f:
        log += f.read()
    tmp = findall(r" Speculative front-running rate: \d+.\d+% \((\d+)\/(\d+)\) ", log)
    tmp = [(s, t) for s, t in tmp]
    succ_num = 0
    total_num = 0
    for s, t in tmp:
        succ_num += int(s)
        total_num += int(t)
    return succ_num, total_num


@task
def create(ctx, nodes=2):
    """Create a testbed"""
    try:
        InstanceManager.make().create_instances(nodes)
    except BenchError as e:
        Print.error(e)


@task
def destroy(ctx):
    """Destroy the testbed"""
    try:
        InstanceManager.make().terminate_instances()
    except BenchError as e:
        Print.error(e)


@task
def start(ctx, max=2):
    """Start at most `max` machines per data center"""
    try:
        InstanceManager.make().start_instances(max)
    except BenchError as e:
        Print.error(e)


@task
def stop(ctx):
    """Stop all machines"""
    try:
        InstanceManager.make().stop_instances()
    except BenchError as e:
        Print.error(e)


@task
def info(ctx):
    """Display connect information about all the available machines"""
    try:
        InstanceManager.make().print_info()
    except BenchError as e:
        Print.error(e)


@task
def install(ctx):
    """Install the codebase on all machines"""
    try:
        Bench(ctx).install()
    except BenchError as e:
        Print.error(e)


@task
def remote(ctx, debug=False):
    """Run benchmarks on AWS"""
    bench_params = {
        "faults": 3,
        "nodes": [10],
        "workers": 1,
        "collocate": True,
        "rate": [10_000, 110_000],
        "tx_size": 512,
        "duration": 300,
        "runs": 2,
    }
    node_params = {
        "header_size": 1_000,  # bytes
        "max_header_delay": 200,  # ms
        "gc_depth": 50,  # rounds
        "sync_retry_delay": 10_000,  # ms
        "sync_retry_nodes": 3,  # number of nodes
        "batch_size": 500_000,  # bytes
        "max_batch_delay": 200,  # ms
    }
    try:
        Bench(ctx).run(bench_params, node_params, debug)
    except BenchError as e:
        Print.error(e)


@task
def plot(ctx):
    """Plot performance using the logs generated by "fab remote" """
    plot_params = {
        "faults": [0],
        "nodes": [10, 20, 50],
        "workers": [1],
        "collocate": True,
        "tx_size": 512,
        "max_latency": [3_500, 4_500],
    }
    try:
        Ploter.plot(plot_params)
    except PlotError as e:
        Print.error(BenchError("Failed to plot performance", e))


@task
def kill(ctx):
    """Stop execution on all machines"""
    try:
        Bench(ctx).kill()
    except BenchError as e:
        Print.error(e)


@task
def logs(ctx):
    """Print a summary of the logs"""
    try:
        print(LogParser.process("./logs", faults="?").result())
    except ParseError as e:
        Print.error(BenchError("Failed to parse logs", e))
