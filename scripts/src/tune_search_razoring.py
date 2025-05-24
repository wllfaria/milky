import time
import math
import os
import json
import subprocess

from typing import TypedDict, cast

from ax.api.types import TParameterValue
from ax.api.client import Client
from ax.api.configs import RangeParameterConfig


class StatsEntry(TypedDict):
    wins: int
    losses: int
    draws: int
    penta_WW: int
    penta_WD: int
    penta_WL: int
    penta_DD: int
    penta_LD: int
    penta_LL: int


class Stats(TypedDict):
    __root__: dict[str, StatsEntry]


class EngineResult(TypedDict):
    wins: int
    draws: int
    losses: int


def get_engine_names(stats: dict[str, StatsEntry]):
    for key in stats:
        tokens = key.split(" ")

        engine_1 = tokens[0]
        engine_2 = tokens[2]
        return (engine_1, engine_2)

    raise Exception("unreachable")


def aggregate_results():
    config = open("./config.json", "r")
    content = cast(
        dict[str, dict[str, StatsEntry]], json.loads("".join(config.readlines()))
    )

    stats = content["stats"]

    (engine_1, engine_2) = get_engine_names(stats)
    scores: dict[str, EngineResult] = {
        engine_1: {"wins": 0, "draws": 0, "losses": 0},
        engine_2: {"wins": 0, "draws": 0, "losses": 0},
    }

    for matchup, entry in stats.items():
        e1, _, e2 = matchup.partition(" vs ")

        if e1 == engine_1 and e2 == engine_2:
            scores[engine_1]["wins"] += entry["wins"]
            scores[engine_1]["draws"] += entry["draws"]
            scores[engine_1]["losses"] += entry["losses"]

            scores[engine_2]["wins"] += entry["losses"]
            scores[engine_2]["draws"] += entry["draws"]
            scores[engine_2]["losses"] += entry["wins"]
        elif e1 == engine_2 and e2 == engine_1:
            scores[engine_2]["wins"] += entry["wins"]
            scores[engine_2]["draws"] += entry["draws"]
            scores[engine_2]["losses"] += entry["losses"]

            scores[engine_1]["wins"] += entry["losses"]
            scores[engine_1]["draws"] += entry["draws"]
            scores[engine_1]["losses"] += entry["wins"]
        else:
            raise ValueError(f"Unexpected matchup key: {matchup}")

    return scores


def elo_derivations(scores: dict[str, EngineResult]):
    derivations: dict[str, tuple[float, float]] = {}

    for engine, results in scores.items():
        games = results["wins"] + results["draws"] + results["losses"]
        wins = results["wins"] / games
        draws = results["draws"] / games
        score = wins + 0.5 * draws
        elo = -400 * math.log10(1 / score - 1)

        p = score
        stddev = math.sqrt(p * (1 - p) / games)
        sem = abs(-400 * math.log10(1 / (p + stddev) - 1) - elo)

        derivations[engine] = (elo, sem)

    return derivations


def run_fastchess(depth: TParameterValue, margin: TParameterValue, engine: str):
    env = os.environ.copy()
    env["DEPTH"] = str(depth)
    env["MARGIN"] = str(margin)

    cmd = [
        "fastchess",
        "-engine",
        "name=milky-v0.3.0",
        "cmd=../target/release/milky",
        "-engine",
        "name=milky-v0.2.0",
        "cmd=./versions/milky-v0.2.0",
        "-each",
        "tc=2+0.2",
        "proto=uci",
        "-rounds",
        "50",
        "-recover",
        "-concurrency",
        "6",
        "-pgnout",
        "file=games.pgn",
        "-openings",
        "file=./data/openings.epd",
        "format=epd",
    ]

    result = subprocess.run(cmd, env=env)
    result.check_returncode()

    scores = aggregate_results()
    derivations = elo_derivations(scores)
    return derivations[engine]


def run_optimization():
    client = Client()

    parameters = [
        RangeParameterConfig(name="depth", parameter_type="int", bounds=(1, 10)),
        RangeParameterConfig(name="margin", parameter_type="int", bounds=(1, 2000)),
    ]

    client.configure_experiment(parameters=parameters)
    client.configure_optimization(objective="elo")

    total_start = time.perf_counter()

    for _ in range(10):
        trials = client.get_next_trials(max_trials=1)

        for trial_index, parameters in trials.items():
            depth = parameters["depth"]
            margin = parameters["margin"]

            start = time.perf_counter()

            elo, sem = run_fastchess(depth, margin, "milky-v0.3.0")
            raw_data = {"elo": (elo, sem)}
            _ = client.complete_trial(trial_index=trial_index, raw_data=raw_data)

            elapsed = time.perf_counter() - start
            print(
                f"Completed trial {trial_index} with parameters depth={depth}, margin={margin} "
                + f"and data {raw_data['elo']} in {elapsed:.2f} seconds"
            )

    total_elapsed = time.perf_counter() - total_start
    best_parameters, prediction, _index, _name = client.get_best_parameterization()
    print("Best Parameters:", best_parameters)
    print("Prediction (mean, variance):", prediction)
    print(f"Total optimization run time: {total_elapsed:.2f} seconds")


if __name__ == "__main__":
    run_optimization()
