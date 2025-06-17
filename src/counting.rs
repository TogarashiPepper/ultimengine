use std::cmp::{max, min};

use crate::{
    board::{Board, Slot, State},
    game::Game,
    moves::{Move, legal_moves},
};

pub const MAX_DEPTH: u8 = 9;

pub fn alpha_beta(game: &Game) -> (i32, Move) {
    let mut mv = Move {
        game: 99,
        index: 99,
    };

    let scr = _alpha_beta::<true>(game, &mut mv, 0, i32::MIN, i32::MAX);

    (scr, mv)
}

fn _alpha_beta<const IS_MAX: bool>(
    game: &Game,
    choice: &mut Move,
    depth: u8,
    mut alp: i32,
    mut bet: i32,
) -> i32 {
    if depth == MAX_DEPTH || game.state != State::Undecided {
        return score_game(game, if IS_MAX { Slot::O } else { Slot::X });
    }

    let mut lgs = legal_moves(game);
    let len = lgs.len();
    if IS_MAX {
        let mut value = i32::MIN;

        // Scoring the games to sort them is costly
        // but alpha-beta pruning benefits so much from
        // a sorted list that it's worth it (see sortdepthanalysis.txt)
        if depth <= 3 {
            lgs.sort_by(|a, b| {
                let asim = game.sim_move(*a, Slot::X).unwrap();
                let bsim = game.sim_move(*b, Slot::X).unwrap();

                score_game(&bsim, Slot::X).cmp(&score_game(&asim, Slot::X))
            });
        }

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::X).unwrap();
            let eval = _alpha_beta::<false>(
                &sim,
                choice,
                min(
                    depth + (len >= 2) as u8 + 2 * (sim.active == 9) as u8,
                    MAX_DEPTH,
                ),
                alp,
                bet,
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

        if depth <= 3 {
            lgs.sort_by(|a, b| {
                let asim = game.sim_move(*a, Slot::X).unwrap();
                let bsim = game.sim_move(*b, Slot::X).unwrap();

                score_game(&asim, Slot::X).cmp(&score_game(&bsim, Slot::X))
            });
        }

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::O).unwrap();
            let eval = _alpha_beta::<true>(
                &sim,
                choice,
                min(
                    depth + (len >= 2) as u8 + (sim.active == 9) as u8,
                    MAX_DEPTH,
                ),
                alp,
                bet,
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
#[inline]
pub fn one_away_x(line: [Slot; 3]) -> bool {
    use Slot::{Empty, X};

    matches!(line, [X, X, Empty] | [Empty, X, X] | [X, Empty, X])
}

#[inline]
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
            scr += 100;
        } else if st == State::Lost {
            scr -= 100;
        }
    }

    if game.active == 9 && turn == Slot::X {
        scr -= scr / 3;
    } else if game.active == 9 && turn == Slot::O {
        scr += scr / 3;
    }

    scr
}

// Takes a `Board` and returns a "score" for how good it is for `X`
#[inline]
pub fn score(board: Board, turn: Slot) -> i32 {
    let mut score = 0;

    // We like corners because they open up the ability to make diagonals,
    // something which top mid, bot mid and the two sides dont let us do
    for corner in [board[0], board[2], board[6], board[8]] {
        if corner == Slot::X {
            score += 1;
        } else if corner == Slot::O {
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

#[inline]
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
