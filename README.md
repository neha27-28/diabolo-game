#  Crablo

A tiny isometric dungeon crawler built from scratch in Rust - because why not make a game right?

You're a little knight. There are goblins. They want to hurt you. Gold is scattered around. You know what to do.

---

##  Play it now

**[neha27-28.github.io/diabolo-game](https://neha27-28.github.io/diabolo-game)**

No install. No sign-up. Just click and play in your browser.

---

## How to play

- **Click anywhere** on the floor to move there
- **Click on a goblin** to attack it (takes 3 hits to kill one)
- **Walk over gold coins** to collect them (+100 points each)
- Kill all goblins before they drain your HP to win
- Goblins chase you and deal 5 damage per hit - don't let them surround you!

---

## What's in the game

- Isometric tile map with walls and obstacles
- Click-to-move pathfinding using BFS (Breadth-First Search)
- 3 goblins with their own AI - they actually chase you (manhattan distance)
- Floating damage numbers when you hit or get hit
- Gold coins to collect around the map
- HP and score display
- Victory and Game Over screens with your final score
- Works on desktop and mobile (tap to move!)

---

## Built with

- **Rust** - the whole game logic
- **macroquad** - handles the window, drawing, and input
- **WebAssembly** - so it runs in the browser without any install
- **GitHub Actions** - auto-builds and deploys every time we push code
- **GitHub Pages** - free hosting

---

## Run it locally

You'll need Rust installed. If you don't have it: [rustup.rs](https://rustup.rs)

```bash
git clone https://github.com/neha27-28/diabolo-game
cd diabolo-game
cargo run
```

To build the web version yourself:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

---

## Why I made this

I'm learning and wanted to build something visual instead of just doing exercises. 

---

## What I learned (still a newbie)

- How isometric coordinates work (converting between tile and screen space)
- BFS pathfinding and how to use it for both player movement and enemy AI
- How to compile Rust to WebAssembly and host it on GitHub Pages
- How GitHub Actions can automate the whole build + deploy pipeline
- How to draw stuff with just lines, circles, and triangles 


