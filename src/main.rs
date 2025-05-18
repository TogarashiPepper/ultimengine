mod board;

use std::io::Write;

use board::{Board, Slot, Game};

fn parse_move(move: &str) -> (usize, usize) {

}

fn main() {
    let mut game = Game::new();

    loop {
        println!("{}", game.print());

        print!("Enter your move (ex. A5): ");
        std::io::stdout().flush().unwrap();

        let mut user_move = String::new();
        std::io::stdin().read_line(&mut user_move).unwrap();        
        
        print!("\x1B[2J\x1B[1;1H");
    }
}
