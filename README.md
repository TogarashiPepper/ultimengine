# Ultimate Tic Tac Toe Engine
An engine for Ultimate TTT using bitboards & minimax alpha-beta pruning. Here's an example of it in action:


![output](https://github.com/user-attachments/assets/45b0b390-519c-4687-908a-1c6e0bb8013f)

# Usage
First clone the repository:
```bash
git clone https://github.com/TogarashiPepper/ultimengine.git
```
Then compile the project (this may take quite a bit as many optimizations are turned on):
```bash
cargo build --release
```
Then execute the resulting binary and play to your heart's content:
```bash
./target/release/ultimengine
```
Moves take the form `{game}{idx}` (i.e `a2`). The games are lettered `a..=i` and the idx is `1..=9`, you may omit the game if a specific game is active. (i.e `1`)
