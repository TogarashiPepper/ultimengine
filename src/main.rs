mod board;
mod counting;
mod game;
mod moves;

use board::Slot;
use counting::won_for;
use game::Game;
use moves::parse_move;

fn redraw(game: &Game) {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", game.print());
}

const DBG_INFO: bool = true;

fn main() {
    let mut game = Game::test();
    let mut mov = String::new();
    let mut rng = rand::rng();

    // Let
    let mut void = String::new();
    let mut stdin = std::io::stdin();

    loop {
        redraw(&game);

        if DBG_INFO {
            use std::fmt::Write;

            let mut bf = String::new();

            write!(bf, "won: ").unwrap();
            for b in game.boards {
                write!(bf, "{:?}, ", won_for(b, Slot::X)).unwrap();
            }

            println!("{bf}");
        }

        print!(
            "Enter your move (ex. a5, active board: {}): ",
            if game.active == 9 {
                ' '
            } else {
                (game.active as u8 + b'a') as char
            }
        );

        use std::io::Write;
        std::io::stdout().flush().unwrap();

        mov.clear();
        stdin.read_line(&mut mov).unwrap();

        let mv = parse_move(mov.trim(), game.active).and_then(|mv| game.make_move(mv, Slot::O));
        if let Err(e) = mv {
            println!("\x1b[0;31m{e} (press enter to continue)\x1b[0m");

            // TODO: make it more efficient?
            stdin.read_line(&mut void).unwrap();

            continue;
        }

        // redraw(&game);

        // // Eng move:
        // let rn = rng.random_range(0..9);
        // game.boards[game.active][rn] = Slot::X;
        // game.active = rn;
        //
        // std::thread::sleep(Duration::from_secs(1));
    }
}
