#![allow(dead_code)]
use ::std::collections::VecDeque;
use macroquad::prelude::*;
use std::{collections::btree_map::Entry::Occupied, iter::once};

const MAP: usize = 20;
const T_SIZE: (f32, f32) = (32., 16.);

enum AppState {
    Menu,
    Playing,
    GameOver,
}
//derive macro to automatically implement the Copy, Clone and PartialEq traits for the AppState enum
#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Wall,
    Floor,
}

//Monsters
struct Monster {
    x: usize,
    y: usize,
    hp: i32,
    cd: f32,
}

//struct for floating text
//dmg theplayer
struct DmgText {
    x: f32,
    y: f32,
    dmg: i32,
    life: f32, //lifetime of the floating text
}

//Math helper
fn to_screen(x: usize, y: usize, cam: (f32, f32)) -> (f32, f32) {
    (
        //convert the x and y coordinates to screen coordinates using the tile size and camera position
        (x as f32 - y as f32) * T_SIZE.0 + cam.0,
        (x as f32 + y as f32) * T_SIZE.1 + cam.1,
    )
}

fn to_tile(sx: f32, sy: f32, cam: (f32, f32)) -> (usize, usize) {
    let (ax, ay) = (sx - cam.0, sy - cam.1);
    (
        ((ax / T_SIZE.0 + ay / T_SIZE.1) / 2.) as usize, //convert the screen coordinates back to tile coordinates using the tile size and camera position
        ((ay / T_SIZE.1 - ax / T_SIZE.0) / 2.) as usize,
    )
}

//calculate distance Manhattan distance
fn dist(p1: (usize, usize), p2: (usize, usize)) -> i32 {
    (p1.0 as i32 - p2.0 as i32).abs() + (p1.1 as i32 - p2.1 as i32).abs()
}

//pathfinding algo- bfs
fn bfs(
    map: &[[Tile; MAP]; MAP],
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut q = VecDeque::from([start]);
    //keeping track of the visited nodes so that the hero doesnt visit it again, and we could calculate the shortest path
    let mut visited = [[false; MAP]; MAP];
    //checking it
    visited[start.1][start.0] = true;

    let mut parent = [[None; MAP]; MAP];

    //if c is not start, then push it into the parent
    while let Some(curr) = q.pop_front() {
        if curr == goal {
            let mut path = vec![];
            let mut c = goal;
            while c != start {
                path.push(c);
                c = parent[c.1][c.0].unwrap();
            }
            //player starts again
            path.reverse();
            return path;
        }

        //check for the close neighbours- left, right, up, down
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = ((curr.0 as i32 + dx) as usize, (curr.1 as i32 + dy) as usize);

            //user can walk only on the floor
            if nx < MAP && ny < MAP && !visited[ny][nx] && map[ny][nx] == Tile::Floor {
                // Mark this neighboring tile as visited so it is not processed again
                visited[ny][nx] = true;

                /* Store the current tile as the parent of this neighbor.
                This allows us to reconstruct the shortest path later.*/
                parent[ny][nx] = Some(curr);

                // Add the neighbor to the back of the queue so it will be explored
                // in FIFO order during the BFS traversal.
                q.push_back((nx, ny));
            }
        }
    }

    vec![]
}

//draw hero and monsters
//IMPROVED THE LOOK OF THE HERO
fn draw_stickman(x: usize, y: usize, cam: (f32, f32), enemy: bool) {
    let (sx, mut sy) = to_screen(x, y, cam);
    sy += 16.;

    //shadow
    draw_ellipse(sx, sy + 3., 14., 6., 0., Color::new(0., 0., 0., 0.18));

    //-----------enemy head----------------
    if enemy {
        //===GOBLIN MONSTER===

        //stuby legs- wide stance

        draw_line(
            sx - 2.,
            sy - 4.,
            sx - 8.,
            sy + 4.,
            3.,
            Color::new(0.1, 0.35, 0.1, 1.),
        );
        draw_line(
            sx + 2.,
            sy - 4.,
            sx + 8.,
            sy + 4.,
            3.,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //feet/claws
        draw_line(
            sx - 8.,
            sy + 4.,
            sx - 13.,
            sy + 2.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 8.,
            sy + 4.,
            sx - 10.,
            sy + 7.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 8.,
            sy + 4.,
            sx + 13.,
            sy + 2.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 8.,
            sy + 4.,
            sx + 10.,
            sy + 7.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //hunched body (wider, squatter than hero)
        draw_ellipse(sx, sy - 14., 10., 14., 0., Color::new(0.13, 0.45, 0.13, 1.));

        //left arm reaching out with claw
        draw_line(
            sx - 10.,
            sy - 20.,
            sx - 20.,
            sy - 10.,
            3.,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //left claw fingers
        draw_line(
            sx - 20.,
            sy - 10.,
            sx - 26.,
            sy - 14.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 20.,
            sy - 10.,
            sx - 25.,
            sy - 8.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 20.,
            sy - 10.,
            sx - 23.,
            sy - 4.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //right arm raised
        draw_line(
            sx + 10.,
            sy - 20.,
            sx + 18.,
            sy - 28.,
            3.,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //right claw fingers
        draw_line(
            sx + 18.,
            sy - 28.,
            sx + 24.,
            sy - 32.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 18.,
            sy - 28.,
            sx + 25.,
            sy - 26.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 18.,
            sy - 28.,
            sx + 22.,
            sy - 22.,
            2.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //big round head
        draw_circle(sx, sy - 16., 12., Color::new(0.13, 0.45, 0.13, 1.));

        //head outline
        draw_circle_lines(sx, sy - 36., 12., 1., Color::new(0.05, 0.2, 0.05, 1.));

        //big pointy ears
        draw_triangle(
            vec2(sx - 12., sy - 40.),
            vec2(sx - 22., sy - 52.),
            vec2(sx - 6., sy - 44.),
            Color::new(0.13, 0.45, 0.13, 1.),
        );

        //right ears
        draw_triangle(
            vec2(sx + 12., sy - 40.),
            vec2(sx + 22., sy - 52.),
            vec2(sx + 6., sy - 44.),
            Color::new(0.13, 0.45, 0.13, 1.),
        );

        //---Glowing red eyes--
        //LEFT
        draw_circle(sx - 5., sy - 38., 3., Color::new(0.9, 0.05, 0.05, 1.));
        //RIGHT
        draw_circle(sx + 5., sy - 38., 3., Color::new(0.9, 0.05, 0.05, 1.));

        //---eyes shine---
        //LEFT
        draw_circle(sx - 4., sy - 39., 1., WHITE);
        //RIGHT
        draw_circle(sx + 6., sy - 39., 1., WHITE);

        //---fangs---
        //LEFT
        draw_line(
            sx - 4.,
            sy - 30.,
            sx - 3.,
            sy - 26.,
            2.,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
        //MIDDLE
        draw_line(
            sx,
            sy - 30.,
            sx,
            sy - 25.,
            2.,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
        //RIGHT
        draw_line(
            sx + 4.,
            sy - 30.,
            sx + 3.,
            sy - 26.,
            2.,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
    } else {
        //cape drawn first- so it is behind the body
        let cape = [
            vec2(sx + 2., sy - 28.),
            vec2(sx + 16., sy - 14.),
            vec2(sx + 12., sy + 2.),
            vec2(sx + 2., sy - 6.),
        ];
        draw_triangle(cape[0], cape[1], cape[2], Color::new(0.75, 0.1, 0.1, 0.88));
        draw_triangle(cape[0], cape[2], cape[3], Color::new(0.75, 0.1, 0.1, 0.88));

        //legs
        draw_line(sx, sy - 8., sx - 6., sy + 2., 2., BLACK);
        draw_line(sx, sy - 8., sx + 6., sy + 2., 2., BLACK);

        //BODY Tunic
        draw_rectangle(
            sx - 5.,
            sy - 28.,
            10.,
            22.,
            Color::new(0.17, 0.43, 0.61, 1.),
        );

        //belt
        draw_line(
            sx - 5.,
            sy - 14.,
            sx + 5.,
            sy - 14.,
            1.5,
            Color::new(0.1, 0.28, 0.42, 1.),
        );

        //left arm going toward shield
        draw_line(sx - 5., sy - 24., sx - 9., sy - 16., 2., BLACK);

        //right arm going toward the sword
        draw_line(sx + 5., sy - 24., sx + 8., sy - 10., 2., BLACK);

        //shield moved furthur to the left
        draw_rectangle(sx - 17., sy - 30., 8., 16., Color::new(0.5, 0.5, 0.5, 1.));
        draw_rectangle(sx - 18., sy - 32., 10., 5., Color::new(0.6, 0.6, 0.6, 1.));

        //sword
        //blade
        draw_line(
            sx + 8.,
            sy - 60.,
            sx + 8.,
            sy - 10.,
            2.,
            Color::new(0.7, 0.7, 0.75, 1.),
        );
        //crossguard
        draw_line(
            sx + 3.,
            sy - 36.,
            sx + 13.,
            sy - 36.,
            3.,
            Color::new(0.55, 0.55, 0.55, 1.),
        );
        //pommel
        draw_circle(sx + 8., sy - 62., 3., Color::new(0.91, 0.77, 0.25, 1.));

        //helmet
        //main dome
        draw_rectangle(
            sx - 7.,
            sy - 52.,
            14.,
            10.,
            Color::new(0.56, 0.57, 0.58, 1.),
        );
        //brow rim
        draw_rectangle(sx - 8., sy - 43., 16., 5., Color::new(0.6, 0.62, 0.63, 1.));
        //top crest
        draw_rectangle(sx - 3., sy - 56., 6., 5., Color::new(0.75, 0.76, 0.77, 1.));

        //helmet outline
        draw_rectangle_lines(sx - 7., sy - 52., 14., 10., 1., BLACK);
        draw_rectangle_lines(sx - 8., sy - 43., 16., 5., 1., BLACK);

        //neck_head (skin)
        draw_circle(sx, sy - 38., 7., Color::new(0.96, 0.84, 0.63, 1.));
    }
}

//drawing the walls
fn draw_wall(x: usize, y: usize, cam: (f32, f32)) {
    //transition from tile coordinates to screen coordinates
    let (sx, sy) = to_screen(x, y, cam);
    //v stands for vector, and it is used to create a vector of points that define the shape of the wall
    let v = [
        vec2(sx, sy - 40.),
        vec2(sx + 32., sy - 24.),
        vec2(sx, sy - 8.),
        vec2(sx - 32., sy - 24.),
        vec2(sx + 32., sy),
        vec2(sx, sy + 16.),
        vec2(sx - 32., sy),
    ];

    //colors for the wall, using the Color struct from macroquad
    let colors = [
        Color::new(0.8, 0.8, 0.8, 1.),
        Color::new(0.5, 0.5, 0.5, 1.),
        Color::new(0.6, 0.6, 0.6, 1.),
    ];

    //draw the faces
    //draw the triangles that make up the wall using the points in the vector v and the colors in the colors array
    draw_triangle(v[0], v[1], v[2], colors[0]);
    draw_triangle(v[0], v[2], v[3], colors[0]);
    draw_triangle(v[1], v[4], v[5], colors[1]);
    draw_triangle(v[1], v[5], v[2], colors[1]);
    draw_triangle(v[3], v[2], v[5], colors[2]);
    draw_triangle(v[3], v[5], v[6], colors[2]);

    //draw outline
    //draw the outline of the wall by connecting the points in the vector v
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (2, 5), (3, 6)] {
        //draw a line between the points v[a] and v[b] with a thickness of 1 and color black
        draw_line(v[a].x, v[a].y, v[b].x, v[b].y, 1., BLACK);
    }
}

//using struct to define the objects in the game
struct Game {
    map: [[Tile; MAP]; MAP],
    cam: (f32, f32),
    px: usize,
    py: usize,
    //target: Option<(usize, usize)>,
    //before we had only the target but now we have path
    path: Vec<(usize, usize)>,
    //cd is cooldown timer-.15s
    player_cd: f32,
    monsters: Vec<Monster>,
    texts: Vec<DmgText>,
    hp: i32,
    gold: Vec<(usize, usize)>,
    score: i32,
}

//creating a class
impl Game {
    fn new() -> Self {
        let mut map = [[Tile::Floor; MAP]; MAP];
        //0 till map size, and fill the map with walls
        for i in 0..MAP {
            //setting the walls on the edges of the map
            map[0][i] = Tile::Wall;

            map[MAP - 1][i] = Tile::Wall;
            map[i][0] = Tile::Wall;
            map[i][MAP - 1] = Tile::Wall;
        }

        //add obstacles in the map, and add 3 pillars in the middle of the map
        for (x, y) in [(5, 5), (6, 5), (12, 10)] {
            map[y][x] = Tile::Wall;
        }
        //returning the game object with the map and camera position
        Game {
            map,
            //setting the camera position to the center of the screen
            cam: (screen_width() / 2.0, 50.),
            px: 2,
            py: 2,
            path: vec![],
            player_cd: 0.,
            monsters: vec![
                Monster {
                    x: 8,
                    y: 8,
                    hp: 30,
                    cd: 0.,
                },
                Monster {
                    x: 12,
                    y: 4,
                    hp: 30,
                    cd: 0.,
                },
                Monster {
                    x: 15,
                    y: 12,
                    hp: 30,
                    cd: 0.,
                },
            ],
            texts: vec![],
            hp: 100,
            score: 0,
            gold: vec![(3, 3), (10, 2), (16, 5), (6, 14), (17, 17)],
        }
    }

    //_dt is data type of 32 bit integer
    fn update(&mut self, dt: f32) -> bool {
        //if player has 0 pt, return true, the game is over
        if self.hp <= 0 || self.monsters.is_empty() {
            return true;
        }

        //update text animations
        self.texts.retain_mut(|t| {
            t.life -= dt;
            t.y -= 20. * dt;
            t.life > 0.
        });

        //mouse button pressed, and get the mouse position
        if is_mouse_button_pressed(MouseButton::Left) {
            //get the mouse position and convert it to tile coordinates
            let (mx, my) = mouse_position();
            //convert the mouse position to tile coordinates using the camera position
            let (tx, ty) = to_tile(mx, my, self.cam);

            /* Check if the clicked tile is within the map boundaries and is walkable.
            If it is, compute the shortest path from the player's current position
            to the clicked destination using Breadth-First Search (BFS).*/
            if tx < MAP && ty < MAP && self.map[ty][tx] == Tile::Floor {
                self.path = bfs(&self.map, (self.px, self.py), (tx, ty));
            }
        }

        // Handle player movement along the computed path.
        if !self.path.is_empty() {
            // Decrease the movement cooldown by the time elapsed since the last frame.
            self.player_cd -= dt;

            // Once the cooldown expires, the player can move to the next tile.
            if self.player_cd <= 0.0 {
                // Reset the cooldown before the next movement.
                self.player_cd = 0.15;

                let (nx, ny) = self.path[0];

                //combat logic for the player
                /*
                --iter() - Goes through all the monsters one by one.
                --position(...) - Finds the index of the first monster that satisfies the condition.
                --|m| m.x == nx && m.y == ny - Checks whether the monster is at (nx, ny).
                --if let Some(i) - If such a monster exists, its index is stored in i. If no monster is found (None), the if block is skipped.
                */
                if let Some(i) = self.monsters.iter().position(|m| m.x == nx && m.y == ny) {
                    //attack
                    self.damage_monster(i, 10);

                    //stop moving
                    self.path.clear();
                } else {
                    //move
                    self.path.remove(0);
                    self.px = nx;
                    self.py = ny;

                    if let Some(i) = self.gold.iter().position(|&g| g == (self.px, self.py)) {
                        self.gold.remove(i);
                        self.score += 100;

                        //spawn a green text
                        let (sx, sy) = to_screen(self.px, self.py, self.cam);
                        self.texts.push(DmgText {
                            x: (sx),
                            y: (sy - 4.),
                            dmg: (-100),
                            life: (1.),
                        });
                    }
                }
            }
        }

        //Monster logic
        //calculate the occupied spots so enemies dont stack
        //we have all the monsters
        //mapped thru each monster
        //chain them once with the position of the player
        //collects everything inside a new vector
        //to have vector of all the occupied tiles
        let occupied: Vec<_> = self
            .monsters
            .iter()
            .map(|m| (m.x, m.y))
            .chain(std::iter::once((self.px, self.py)))
            .collect();

        for i in 0..self.monsters.len() {
            self.monsters[i].cd -= dt;
            if self.monsters[i].cd <= 0. {
                self.monsters[i].cd = 1.0; //slower than the player

                let (mx, my) = (self.monsters[i].x, self.monsters[i].y);
                let d = dist((mx, my), (self.px, self.py));

                //depending on the distance we will have diff things
                // If the monster is next to the player, attack the player.
                if d == 1 {
                    // Reduce the player's health.
                    self.hp -= 5;

                    // Convert the player's tile position to screen coordinates.
                    let (sx, sy) = to_screen(self.px, self.py, self.cam);

                    // Add a floating damage text above the player.
                    self.texts.push(DmgText {
                        x: sx,
                        y: sy - 40.,
                        dmg: 5,
                        life: 1.,
                    });
                } else {
                    //chase the player
                    let path = bfs(&self.map, (mx, my), (self.px, self.py));
                    if path.len() > 1 && !occupied.contains(&path[0]) {
                        self.monsters[i].x = path[0].0;
                        self.monsters[i].y = path[0].1;
                    }
                }
            }
        }

        false
    }

    //helper to damage monsters
    fn damage_monster(&mut self, idx: usize, amount: i32) {
        //reduce life of monsters
        self.monsters[idx].hp -= amount;

        //spawn text
        //// Convert the monster's tile position to screen coordinates.
        let (sx, sy) = to_screen(self.monsters[idx].x, self.monsters[idx].y, self.cam);

        //// Add a floating damage text that appears above the monster.
        self.texts.push(DmgText {
            x: (sx),
            y: (sy - 40.),
            dmg: (amount),
            life: (1.),
        });

        //kill logic
        //// If the monster has no health left, remove it from the game.
        if self.monsters[idx].hp <= 0 {
            self.monsters.remove(idx);

            //50 bonus if we kill a monster
            self.score += 50;
        }
    }

    //creating a function to draw the game state
    fn draw(&self) {
        //populating the map with walls and floors
        for y in 0..MAP {
            for x in 0..MAP {
                if self.map[y][x] == Tile::Wall {
                    //draw the wall at the given x and y coordinates, using the camera position to adjust the screen coordinates
                    draw_wall(x, y, self.cam);
                } else {
                    //draw a dot with circle shape
                    if self.gold.contains(&(x, y)) {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy + 16., 6., GOLD);
                    } else {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy + 16., 2., LIGHTGRAY);
                    }
                }
            }
        }

        //draw the path
        for (px, py) in &self.path {
            let (sx, sy) = to_screen(*px, *py, self.cam);
            draw_circle(sx, sy + 16., 4., GOLD);
        }

        //draw the player character at the given x and y coordinates, using the camera position to adjust the screen coordinates
        draw_stickman(self.px, self.py, self.cam, false);

        //draw monsters
        for m in &self.monsters {
            draw_stickman(m.x, m.y, self.cam, true);
        }

        //draw floating text
        for t in &self.texts {
            if t.dmg < 0 {
                draw_text(&format!("+{}", -t.dmg), t.x, t.y, 20., GREEN);
            } else {
                draw_text(&format!("-{}", t.dmg), t.x, t.y, 20., RED);
            }
        }

        //HUD-heads on display
        draw_text(
            &format!("HP: {}", self.hp),
            20.,
            screen_height() - 40.,
            30.,
            BLACK,
        );
        draw_text(
            &format!("SCORE: {}", self.score),
            20.,
            screen_height() - 70.,
            30.,
            BLACK,
        );
    }
}

//title of the game window
#[macroquad::main("Crablo")]

async fn main() {
    let mut game = Game::new();
    let mut state = AppState::Menu;

    loop {
        clear_background(WHITE);

        match state {
            AppState::Menu => {
                draw_text("Menu- Enter to start", 100., 100., 40., BLACK);

                //if the user presses enter, change the state to playing
                if is_key_pressed(KeyCode::Enter) {
                    game = Game::new();
                    state = AppState::Playing;
                }
            }

            //if the state is playing, update the game and check if the game is over
            AppState::Playing => {
                if game.update(get_frame_time()) {
                    state = AppState::GameOver;
                }
                //draw the game state
                game.draw();
            }

            //if the state is game over, draw the game over screen
            AppState::GameOver => {
                game.draw();
                draw_rectangle(
                    0.,
                    0.,
                    screen_width(),
                    screen_height(),
                    Color::new(1., 1., 1., 0.7),
                );

                //Victory vs defeat logic
                let (msg, col) = if game.hp > 0 {
                    ("VICTORY", GOLD)
                } else {
                    ("GAME OVER", RED)
                };

                draw_text(
                    msg,
                    screen_width() / 2. - 100.,
                    screen_height() / 2.,
                    60.,
                    col,
                );

                draw_text(
                    &format!("Final Score :{}", game.score),
                    screen_width() / 2. - 80.,
                    screen_height() / 2. + 50.,
                    30.,
                    BLACK,
                );

                draw_text(
                    "Enter to reset",
                    screen_width() / 2.,
                    -80.,
                    screen_height() / 2. + 90.,
                    GRAY,
                );

                if is_key_pressed(KeyCode::Enter) {
                    state = AppState::Menu
                }
            }
        }
        // wait for the next frame to be drawn
        next_frame().await;
    }
}
