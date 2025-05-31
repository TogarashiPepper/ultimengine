mod board;
use rand::Rng;
use std::{io::Write, time::Duration};

use board::{Game, Slot, full};

fn parse_move(movestr: &str) -> Result<(usize, usize), &'static str> {
    if movestr.len() != 2 {
        return Err("Move string too long");
    }

    // Purposefully Invalid sentinel
    let mut mov = (9, 9);
    let [row, col] = movestr.as_bytes() else {
        unreachable!()
    };

    if ('a'..='i').contains(&char::from(*row).to_ascii_lowercase()) {
        mov.0 = *row as usize - 'a' as usize;
    }

    if char::from(*col).is_ascii_digit() {
        mov.1 = *col as usize - '0' as usize - 1;
    }

    if mov.0 == 9 || mov.1 == 9 {
        return Err("invalid row or column");
    }

    Ok(mov)
}

fn redraw(game: &Game) {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", game.print());
}

fn main() {
    let mut game = Game::new();
    let mut mov = String::new();
    let mut rng = rand::rng();

    loop {
        redraw(&game);

        print!(
            "Enter your move (ex. a5, active board: {}): ",
            if game.active == 9 {
                ' '
            } else {
                (game.active as u8 + b'a') as char
            }
        );
        std::io::stdout().flush().unwrap();

        mov.clear();
        std::io::stdin().read_line(&mut mov).unwrap();

        let (game_num, idx) = parse_move(mov.trim()).unwrap();
        if game.active == 9 {
            game.active = game_num;
        }

        if game.active != game_num {
            println!("must play in the active board");
            std::thread::sleep(Duration::from_secs(1));

            continue;
        }

        if game.boards[game_num][idx] != Slot::Empty {
            println!("{} is not empty", mov.trim());
            std::thread::sleep(Duration::from_secs(1));

            continue;
        }

        game.boards[game_num][idx] = Slot::O;
        game.active = idx;
        redraw(&game);

        // Eng move:
        let rn = rng.random_range(0..9);
        game.boards[game.active][rn] = Slot::X;
        game.active = rn;

        std::thread::sleep(Duration::from_secs(1));
    }
}
