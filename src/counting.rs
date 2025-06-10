use crate::{board::{Board, Slot}, game::Game};

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
    // Favor moves that make us one away or moves that win
    // defavor moves that make them one away or win
    // defavor moves that make game.active == 9

    todo!()
}

// Takes a `Board` and returns a "score" for how good it is for `X`
// `side` must not be `Slot::Empty`
pub fn score(board: Board) -> i32 {
    let mut score = 0;

    for row in board.rows() {
        if one_away_x(row) {
            score = 100;
        }

        if one_away_o(row) {
            score = -100;
        }
    }

    for column in board.columns() {
        if one_away_x(column) {
            score = 100;
        }

        if one_away_o(column) {
            score = -100;
        }
    }

    for diag in board.diags() {
        if one_away_x(diag) {
            score = 100;
        }

        if one_away_o(diag) {
            score = -100;
        }
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

