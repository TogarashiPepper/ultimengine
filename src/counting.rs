use crate::board::{Board, Slot};

// TODO: consolidate functions into one
pub fn one_away_x(line: [Slot; 3]) -> bool {
    use Slot::{Empty, X};

    matches!(line, [X, X, Empty] | [Empty, X, X] | [X, Empty, X])
}

pub fn one_away_o(line: [Slot; 3]) -> bool {
    use Slot::{Empty, O};

    matches!(line, [O, O, Empty] | [Empty, O, O] | [O, Empty, O])
}

// Takes a `Board` and returns a "score" for how good it is for `X`
// `side` must not be `Slot::Empty`
pub fn score(board: Board) -> i32 {
    let mut score = 0;

    for row in board.rows() {
        if one_away_x(row) {
            score = i32::MAX;
        }

        if one_away_o(row) {
            score = i32::MIN;
        }
    }

    for column in board.columns() {
        if one_away_x(column) {
            score = i32::MAX;
        }

        if one_away_o(column) {
            score = i32::MIN;
        }
    }

    for diag in board.diags() {
        if one_away_x(diag) {
            score = i32::MAX;
        }

        if one_away_o(diag) {
            score = i32::MIN;
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

