//! This is an old preserved version of `counting`
//! to be used as a benchmark to test if any changes made to `counting`
//! have improved the engines ability or made it worse

use crate::{
    bitboard::BitBoard,
    board::{Slot, State},
    game::Game,
};

pub fn ref_score_game(game: &Game) -> i32 {
    let mut scr = score(game.shrink(), Slot::O) * 100;

    if let Some(last_move) = game.last_move {
        scr += score(game.boards[last_move.game], Slot::X);

        if last_move.index == game.active {
            scr += score(game.boards[game.active], Slot::O);
        }
    }

    if game.active != 9 && game.last_move.map(|m| m.game) != Some(game.active) {
        let sc = score(game.boards[game.active], Slot::O);
        scr += sc;
    }

    for st in game.boards.map(BitBoard::state) {
        if st == State::Won {
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
fn score(board: BitBoard, turn: Slot) -> i32 {
    let mut score = 0;

    // We like corners because they open up the ability to make diagonals,
    // something which top mid, bot mid and the two sides dont let us do
    score += board.corners(Slot::X);

    if turn == Slot::X {
        score += 5 * board.one_aways_x();
    }

    if turn == Slot::O {
        score -= 5 * board.one_aways_x();
    }

    score -= 5 * board.one_aways_o();

    if board.won_by_x() {
        score = 10_000;
    }

    if board.won_by_o() {
        score = -10_000;
    }

    score
}

// TODO: no conversion
pub fn possible_to_win(board: BitBoard) -> bool {
    use Slot::{Empty as E, O, X};
    let board = crate::board::Board::new_with(board.to_arr());

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
