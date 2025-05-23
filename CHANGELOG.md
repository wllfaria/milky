# Changelog

## [0.2.0] - 2025-05-23

### Features

- Null move pruning and using best move on ordering

### Miscellaneous Tasks

- Updating justfile with better release workflow
- Making insta a dev-dependency
- Adding testing files to gitignore

## [0.1.0] - 2025-05-22

### Bug Fixes

- Removing pv search duplication
- Fixing out of time bug playing bad moves

### Features

- Computing leaper piece moves (king, knight, pawns)
- Blockers, attacks and occupancy tables for sliding pieces
- Rook and bishop magic numbers for move generator
- Pre-computed bishop and rook moves
- Parsing FEN strings
- Implementing a way to check attacked squares
- Early pawn move generation
- Generating pawn captures and en_passant
- Generating every possible move from a position
- Encoding moves
- Storing generated moves on a list
- Making and unmaking moves based on validity
- Perft test setup
- Perft tests for correctness on total nodes at various depths
- Implementing initial UCI commands
- Integrating UCI protocol to the engine search
- Simple evaluation function
- Basic alpha beta search
- Quiescence search to not give material away
- Better search with iterative deepening and principal variation
- Implementing principal variation search
- Implementing late move pruning
- Null move pruning with naive conditions
- Transposition table and zobrist hashing
- UCI protocol implementation
- Simple time manager
- Implementing time management
- Penalizing and giving bonus to pawn structures
- Adding bonus and penalties for open files
- Bonus for king safety
- Slightly improved pruning on static evaluation

### Miscellaneous Tasks

- Project setup
- Updating piece printing for better debugging
- Updating readme
- Updating MSRV and adding citation file

### Refactor

- Fixing clippy issues and refactoring a few functions


