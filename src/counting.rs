use crate::{
    board::{Board, Slot, State},
    game::Game,
};

// TODO: consolidate functions into one
pub fn one_away_x(line: [Slot; 3]) -> bool {
    use Slot::{Empty, X};

    matches!(line, [X, X, Empty] | [Empty, X, X] | [X, Empty, X])
}

pub fn one_away_o(line: [Slot; 3]) -> bool {
    use Slot::{Empty, O};

    matches!(line, [O, O, Empty] | [Empty, O, O] | [O, Empty, O])
}

pub fn score_game(game: &Game) -> i32 {
    // TODO: dont make moves that put the enemy in a spot where they could block a 1 away

    let mut scr = score(game.shrink(), Slot::O) * 100;

    if let Some(last_move) = game.last_move {
        scr += score(game.boards[last_move.game], Slot::X);
    }

    if game.active != 9 && game.last_move.map(|m| m.game) != Some(game.active) {
        let sc = score(game.boards[game.active], Slot::O);
        scr += sc;
    }

    for st in game.states {
        if st == State::Won {
            // TODO: better amount,
            // due to issue with following board:
            //  X | X | O
            // -----------
            //    | X |
            // -----------
            //    |   |
            //  computer would move in 4 because it made the score 21, which was higher
            //  than the previous win score of +6 so it wouldnt win the board
            scr += 100;
        } else if st == State::Lost {
            scr -= 100;
        }
    }

    if game.active == 9 {
        scr -= 3;
    }

    scr
}

// Takes a `Board` and returns a "score" for how good it is for `X`
pub fn score(board: Board, turn: Slot) -> i32 {
    let mut score = 0;

    // We like corners because they open up the ability to make diagonals,
    // something which top mid, bot mid and the two sides dont let us do
    for corner in [board[0], board[2], board[6], board[8]] {
        if corner == Slot::X {
            score += 1;
        }
    }

    for row in board.rows() {
        if one_away_x(row) {
            score += 5;
        }

        if one_away_o(row) {
            score += -5;
        }

        if one_away_o(row) && one_away_o(row) {
            if turn == Slot::X {
                score += 5;
            } else {
                score -= 5
            }
        }
    }

    for column in board.columns() {
        if one_away_x(column) {
            score += 5;
        }

        if one_away_o(column) {
            score += -5;
        }

        if one_away_o(column) && one_away_o(column) {
            if turn == Slot::X {
                score += 5;
            } else {
                score -= 5;
            }
        }
    }

    for diag in board.diags() {
        if one_away_x(diag) {
            score += 5;
        }

        if one_away_o(diag) {
            score += -5;
        }

        if one_away_o(diag) && one_away_o(diag) {
            if turn == Slot::X {
                score += 5;
            } else {
                score -= 5;
            }
        }
    }

    if won_for(board, Slot::X) {
        score = 10_000;
    }

    if won_for(board, Slot::O) {
        score = -10_000;
    }

    score
}

pub fn won_for(board: Board, side: Slot) -> bool {
    board
        .rows()
        .into_iter()
        .chain(board.columns())
        .chain(board.diags())
        .any(|line| line == [side; 3])
}

pub fn possible_to_win(board: Board) -> bool {
    use Slot::{Empty as E, O, X};

    board
        .rows()
        .into_iter()
        .chain(board.columns())
        .chain(board.diags())
        .any(|line| {
            [
                [E; 3],
                [X, E, E],
                [E, E, X],
                [X, E, X],
                [O, E, E],
                [E, E, O],
                [O, E, O],
                [X, X, E],
                [E, X, X],
                [O, O, E],
                [E, O, O],
            ]
            .contains(&line)
        })
}
