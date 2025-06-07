mod board;
mod counting;
mod game;

use board::Slot;
use counting::won_for;
use game::Game;

fn redraw(game: &Game) {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", game.print());
}

const DBG_INFO: bool = true;

fn main() {
    let mut game = Game::new();
    let mut mov = String::new();
    let mut rng = rand::rng();

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
        std::io::stdin().read_line(&mut mov).unwrap();

        let mv = game.parse_move(mov.trim()).unwrap();
        game.make_move(mv, Slot::O).unwrap();

        // redraw(&game);

        // // Eng move:
        // let rn = rng.random_range(0..9);
        // game.boards[game.active][rn] = Slot::X;
        // game.active = rn;
        //
        // std::thread::sleep(Duration::from_secs(1));
    }
}
