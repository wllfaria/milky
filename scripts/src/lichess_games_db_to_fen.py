import chess.pgn
import subprocess


class EvenGame:
    fen: str
    opening: str
    eval: float

    def __init__(self, fen: str, opening: str, eval: float):
        self.fen = fen
        self.opening = opening
        self.eval = eval


def convert():
    pgn = open("./lichess_rated_2015-08.pgn")

    stockfish = spawn_stockfish()
    games: list[EvenGame] = []

    seen_openings: set[str] = set()
    eval_threshold = 0.5
    eval_depth = 12

    current_game = chess.pgn.read_game(pgn)
    while current_game is not None:
        opening = current_game.headers.get("Opening")
        if not opening or opening in seen_openings:
            current_game = chess.pgn.read_game(pgn)
            continue

        game_board = current_game.board()
        moves = list(current_game.mainline_moves())[:8]
        for move in moves:
            game_board.push(move)

        game_fen = game_board.fen()
        evaluation = evaluate_position(stockfish, game_fen, eval_depth)
        if not evaluation or abs(evaluation) >= eval_threshold:
            current_game = chess.pgn.read_game(pgn)
            continue

        seen_openings.add(opening)
        games.append(EvenGame(game_fen, opening, evaluation))
        print(f"Appending game {len(games)}: {opening}, eval: {evaluation}")

        current_game = chess.pgn.read_game(pgn)

        if len(games) >= 2000:
            break

    with open("openings.epd", "w") as f:
        for game in games:
            _ = f.write(f"{game.fen}\n")


def spawn_stockfish() -> subprocess.Popen[str]:
    stockfish = subprocess.Popen(
        ["stockfish"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    assert stockfish.stdin is not None
    assert stockfish.stdout is not None

    _ = stockfish.stdin.write("uci\n")
    stockfish.stdin.flush()

    for line in stockfish.stdout:
        if "uciok" in line:
            break

    return stockfish


def evaluate_position(pid: subprocess.Popen[str], fen: str, depth: int):
    assert pid.stdin is not None
    assert pid.stdout is not None

    _ = pid.stdin.write(f"ucinewgame\n")
    _ = pid.stdin.write(f"position fen {fen}\n")
    _ = pid.stdin.write(f"go depth {depth}\n")
    pid.stdin.flush()

    evaluation: float | None = 0

    for line in pid.stdout:
        if "bestmove" in line:
            break
        if "info" in line and "score" in line:
            tokens = line.strip().split()
            if "cp" in tokens:
                idx = tokens.index("cp")
                evaluation = int(tokens[idx + 1]) / 100
            elif "mate" in tokens:
                return None

    return evaluation


if __name__ == "__main__":
    convert()
