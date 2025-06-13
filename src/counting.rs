use std::cmp::{max, min};

use crate::{
    board::{Board, Slot, State},
    game::Game, moves::{legal_moves, Move},
};

pub const MAX_DEPTH: u8 = 9;

pub fn alpha_beta(
    game: &Game,
    choice: &mut Move,
    depth: u8,
    mut alp: i32,
    mut bet: i32,
    is_max: bool,
) -> i32 {
    if depth == MAX_DEPTH || game.state != State::Undecided {
        return score_game(game, if is_max { Slot::O } else { Slot::X });
    }

    if is_max {
        let mut value = i32::MIN;
        let lgs = legal_moves(game);

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::X).unwrap();
            let eval = alpha_beta(
                &sim,
                choice,
                min(depth + 1 + 2 * (sim.active == 9) as u8, MAX_DEPTH),
                alp,
                bet,
                false,
            );

            if eval > value && depth == 0 {
                *choice = legal;
            }
            value = max(value, eval);

            if value >= bet {
                break;
            }
            alp = max(alp, value);
        }

        value
    } else {
        let mut value = i32::MAX;
        let lgs = legal_moves(game);

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::O).unwrap();
            let eval = alpha_beta(
                &sim,
                choice,
                min(depth + 1 + (sim.active == 9) as u8, MAX_DEPTH),
                alp,
                bet,
                true,
            );

            if eval < value && depth == 0 {
                *choice = legal;
            }
            value = min(value, eval);

            if value <= alp {
                break;
            }
            bet = min(bet, value);
        }

        value
    }
}




// TODO: consolidate functions into one
pub fn one_away_x(line: [Slot; 3]) -> bool {
    use Slot::{Empty, X};

    matches!(line, [X, X, Empty] | [Empty, X, X] | [X, Empty, X])
}

pub fn one_away_o(line: [Slot; 3]) -> bool {
    use Slot::{Empty, O};

    matches!(line, [O, O, Empty] | [Empty, O, O] | [O, Empty, O])
}

pub fn score_game(game: &Game, turn: Slot) -> i32 {
    let mut scr = score(game.shrink(), turn) * 100;

    for brd in game.boards {
        scr += score(brd, turn) / 4;
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

    if game.active == 9 && turn == Slot::X {
        scr -= scr / 3;
    }
    else if game.active == 9 && turn == Slot::O {
        scr += scr / 3;
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
        else if corner == Slot::O {
            score -= 1;
        }
    }

    for line in board
        .rows()
        .into_iter()
        .chain(board.columns())
        .chain(board.diags())
    {
        if one_away_x(line) && turn == Slot::X {
            score += 6;
        }

        if one_away_o(line) && turn == Slot::O {
            score -= 6;
        }

        if one_away_o(line) && turn == Slot::X {
            score += 3;
        }

        if one_away_x(line) && turn == Slot::O {
            score -= 3;
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
