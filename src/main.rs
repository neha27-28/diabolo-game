#![allow(dead_code)]
use ::std::collections::VecDeque;
use macroquad::prelude::*;

//use std::{collections::btree_map::Entry::Occupied, iter::once};

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

/*===========SCALING HELPERS======================== */

//RETUrns 1.0 on the desktop(nochange) but changes to less than 1.0 on small screens
fn calc_scale() -> f32 {
    //isometric diamond's pixel dimensions at scale 1.0
    let map_pixel_w = (MAP as f32) * 2. * T_SIZE.0; //1280px wide
    let map_pixel_h = (MAP as f32) * 2. * T_SIZE.1; //640x tall

    let scale_x = screen_width() / map_pixel_w;
    let scale_y = screen_height() / map_pixel_h;

    //use smaller axis so the whole map fits
    // .min(1.0) means we never scale up on desktop- only shrink on mobile
    scale_x.min(scale_y).min(1.0)
}

// _+==== calculate camera dynamically based on screen size so that it is centered =============
fn calc_cam(scale: f32) -> (f32, f32) {
    //isometric diamond is 2*MAP tiles wide and 2*MAP tiles tall
    let map_pixel_w = (MAP as f32) * 2. * T_SIZE.0 * scale; //1280px
    let map_pixel_h = (MAP as f32) * 2. * T_SIZE.1 * scale; //640px

    (
        (screen_width() - map_pixel_w) / 2. + (MAP as f32) * T_SIZE.0 * scale, //horizontally centerd
        (screen_height() - map_pixel_h) / 2.,                                  //vertically centered
    )
}

/*===========COORDINATE HELPERS======================== */
fn to_screen(x: usize, y: usize, cam: (f32, f32), scale: f32) -> (f32, f32) {
    (
        //convert the x and y coordinates to screen coordinates using the tile size and camera position
        (x as f32 - y as f32) * T_SIZE.0 * scale + cam.0,
        (x as f32 + y as f32) * T_SIZE.1 * scale + cam.1,
    )
}

fn to_tile(sx: f32, sy: f32, cam: (f32, f32), scale: f32) -> (usize, usize) {
    let (ax, ay) = (sx - cam.0, sy - cam.1);
    let tw = T_SIZE.0 * scale;
    let th = T_SIZE.1 * scale;
    (
        ((ax / tw + ay / th) / 2.) as usize, //convert the screen coordinates back to tile coordinates using the tile size and camera position
        ((ay / th - ax / tw) / 2.) as usize,
    )
}

//calculate distance Manhattan distance
fn dist(p1: (usize, usize), p2: (usize, usize)) -> i32 {
    (p1.0 as i32 - p2.0 as i32).abs() + (p1.1 as i32 - p2.1 as i32).abs()
}

/*==============PATHFINDING======================== */
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
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = curr.0 as i32 + dx;
            let ny = curr.1 as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let (nx, ny) = (nx as usize, ny as usize);
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
    }
    vec![]
}

/*==============DRAW CHARS======================== */
//draw hero and monsters
//IMPROVED THE LOOK OF THE HERO
fn draw_stickman(x: usize, y: usize, cam: (f32, f32), scale: f32, enemy: bool) {
    let (sx, mut sy) = to_screen(x, y, cam, scale);
    sy += 16. * scale;

    //shadow
    draw_ellipse(
        sx,
        sy + 3.,
        14.,
        6. * scale,
        0.,
        Color::new(0., 0., 0., 0.18),
    );

    //-----------enemy head----------------
    if enemy {
        //===GOBLIN MONSTER===

        //stuby legs- wide stance
        let s = scale;
        draw_line(
            sx - 2. * s,
            sy - 4. * s,
            sx - 8. * s,
            sy + 4. * s,
            3. * s,
            Color::new(0.1, 0.35, 0.1, 1.),
        );
        draw_line(
            sx + 2. * s,
            sy - 4. * s,
            sx + 8. * s,
            sy + 4. * s,
            3. * s,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //feet/claws
        draw_line(
            sx - 8. * s,
            sy + 4. * s,
            sx - 13. * s,
            sy + 2. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 8. * s,
            sy + 4. * s,
            sx - 10. * s,
            sy + 7. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 8. * s,
            sy + 4. * s,
            sx + 13. * s,
            sy + 2. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 8. * s,
            sy + 4. * s,
            sx + 10. * s,
            sy + 7. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //hunched body (wider, squatter than hero)
        draw_ellipse(
            sx,
            sy - 14. * s,
            10. * s,
            14. * s,
            0.,
            Color::new(0.13, 0.45, 0.13, 1.),
        );

        //left arm reaching out with claw
        draw_line(
            sx - 10. * s,
            sy - 20. * s,
            sx - 20. * s,
            sy - 10. * s,
            3. * s,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //left claw fingers
        draw_line(
            sx - 20. * s,
            sy - 10. * s,
            sx - 26. * s,
            sy - 14. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 20. * s,
            sy - 10. * s,
            sx - 25. * s,
            sy - 8. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx - 20. * s,
            sy - 10. * s,
            sx - 23. * s,
            sy - 4. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //right arm raised
        draw_line(
            sx + 10. * s,
            sy - 20. * s,
            sx + 18. * s,
            sy - 28. * s,
            3. * s,
            Color::new(0.1, 0.35, 0.1, 1.),
        );

        //right claw fingers
        draw_line(
            sx + 18. * s,
            sy - 28. * s,
            sx + 24. * s,
            sy - 32. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 18. * s,
            sy - 28. * s,
            sx + 25. * s,
            sy - 26. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );
        draw_line(
            sx + 18. * s,
            sy - 28. * s,
            sx + 22. * s,
            sy - 22. * s,
            2. * s,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //big round head
        draw_circle(sx, sy - 16. * s, 12. * s, Color::new(0.13, 0.45, 0.13, 1.));

        //head outline
        draw_circle_lines(
            sx,
            sy - 36. * s,
            12. * s,
            1.,
            Color::new(0.05, 0.2, 0.05, 1.),
        );

        //big pointy ears
        draw_triangle(
            vec2(sx - 12. * s, sy - 40. * s),
            vec2(sx - 22. * s, sy - 52. * s),
            vec2(sx - 6. * s, sy - 44. * s),
            Color::new(0.13, 0.45, 0.13, 1.),
        );

        //right ears
        draw_triangle(
            vec2(sx + 12. * s, sy - 40. * s),
            vec2(sx + 22. * s, sy - 52. * s),
            vec2(sx + 6. * s, sy - 44. * s),
            Color::new(0.13, 0.45, 0.13, 1.),
        );

        //---Glowing red eyes--
        //LEFT
        draw_circle(
            sx - 5. * s,
            sy - 38. * s,
            3. * s,
            Color::new(0.9, 0.05, 0.05, 1.),
        );
        //RIGHT
        draw_circle(
            sx + 5. * s,
            sy - 38. * s,
            3. * s,
            Color::new(0.9, 0.05, 0.05, 1.),
        );

        //---eyes shine---
        //LEFT
        draw_circle(sx - 4. * s, sy - 39. * s, 1. * s, WHITE);
        //RIGHT
        draw_circle(sx + 6. * s, sy - 39. * s, 1. * s, WHITE);

        //---fangs---
        //LEFT
        draw_line(
            sx - 4. * s,
            sy - 30. * s,
            sx - 3. * s,
            sy - 26. * s,
            2. * s,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
        //MIDDLE
        draw_line(
            sx,
            sy - 30. * s,
            sx,
            sy - 25. * s,
            2. * s,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
        //RIGHT
        draw_line(
            sx + 4. * s,
            sy - 30. * s,
            sx + 3. * s,
            sy - 26. * s,
            2. * s,
            Color::new(0.95, 0.95, 0.95, 1.),
        );
    } else {
        //----------HERO========
        //cape drawn first- so it is behind the body
        let s = scale;
        let cape = [
            vec2(sx + 2. * s, sy - 28. * s),
            vec2(sx + 16. * s, sy - 14. * s),
            vec2(sx + 12. * s, sy + 2. * s),
            vec2(sx + 2. * s, sy - 6. * s),
        ];
        draw_triangle(cape[0], cape[1], cape[2], Color::new(0.75, 0.1, 0.1, 0.88));
        draw_triangle(cape[0], cape[2], cape[3], Color::new(0.75, 0.1, 0.1, 0.88));

        //legs
        draw_line(sx, sy - 8. * s, sx - 6. * s, sy + 2. * s, 2. * s, BLACK);
        draw_line(sx, sy - 8. * s, sx + 6. * s, sy + 2. * s, 2. * s, BLACK);

        //BODY Tunic
        draw_rectangle(
            sx - 5. * s,
            sy - 28. * s,
            10. * s,
            22. * s,
            Color::new(0.17, 0.43, 0.61, 1.),
        );

        //belt
        draw_line(
            sx - 5. * s,
            sy - 14. * s,
            sx + 5. * s,
            sy - 14. * s,
            1.5 * s,
            Color::new(0.1, 0.28, 0.42, 1.),
        );

        //left arm going toward shield
        draw_line(
            sx - 5. * s,
            sy - 24. * s,
            sx - 9. * s,
            sy - 16. * s,
            2. * s,
            BLACK,
        );

        //right arm going toward the sword
        draw_line(
            sx + 5. * s,
            sy - 24. * s,
            sx + 8. * s,
            sy - 10. * s,
            2. * s,
            BLACK,
        );

        //shield moved furthur to the left
        draw_rectangle(
            sx - 17. * s,
            sy - 30. * s,
            8. * s,
            16. * s,
            Color::new(0.5, 0.5, 0.5, 1.),
        );
        draw_rectangle(
            sx - 18. * s,
            sy - 32. * s,
            10. * s,
            5. * s,
            Color::new(0.6, 0.6, 0.6, 1.),
        );

        //sword
        //blade
        draw_line(
            sx + 8. * s,
            sy - 60. * s,
            sx + 8. * s,
            sy - 10. * s,
            2. * s,
            Color::new(0.7, 0.7, 0.75, 1.),
        );
        //crossguard
        draw_line(
            sx + 3. * s,
            sy - 36. * s,
            sx + 13. * s,
            sy - 36. * s,
            3. * s,
            Color::new(0.55, 0.55, 0.55, 1.),
        );
        //pommel
        draw_circle(
            sx + 8. * s,
            sy - 62. * s,
            3. * s,
            Color::new(0.91, 0.77, 0.25, 1.),
        );

        //helmet
        //main dome
        draw_rectangle(
            sx - 7. * s,
            sy - 52. * s,
            14. * s,
            10. * s,
            Color::new(0.56, 0.57, 0.58, 1.),
        );
        //brow rim
        draw_rectangle(
            sx - 8. * s,
            sy - 43. * s,
            16. * s,
            5. * s,
            Color::new(0.6, 0.62, 0.63, 1.),
        );
        //top crest
        draw_rectangle(
            sx - 3. * s,
            sy - 56. * s,
            6. * s,
            5. * s,
            Color::new(0.75, 0.76, 0.77, 1.),
        );

        //helmet outline
        draw_rectangle_lines(sx - 7. * s, sy - 52. * s, 14. * s, 10. * s, 1. * s, BLACK);
        draw_rectangle_lines(sx - 8. * s, sy - 43. * s, 16. * s, 5. * s, 1. * s, BLACK);

        //neck_head (skin)
        draw_circle(sx, sy - 38. * s, 7. * s, Color::new(0.96, 0.84, 0.63, 1.));
    }
}

/*==============DRAW WALLS======================== */
//drawing the walls
fn draw_wall(x: usize, y: usize, cam: (f32, f32), scale: f32) {
    //transition from tile coordinates to screen coordinates
    let (sx, sy) = to_screen(x, y, cam, scale);
    //v stands for vector, and it is used to create a vector of points that define the shape of the wall
    let s = scale;
    let v = [
        vec2(sx, sy - 40. * s),
        vec2(sx + 32. * s, sy - 24. * s),
        vec2(sx, sy - 8. * s),
        vec2(sx - 32. * s, sy - 24. * s),
        vec2(sx + 32. * s, sy),
        vec2(sx, sy + 16. * s),
        vec2(sx - 32. * s, sy),
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

/*==============GAME STRUCT======================== */
//using struct to define the objects in the game
struct Game {
    map: [[Tile; MAP]; MAP],
    cam: (f32, f32),
    scale: f32,
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
        let scale = calc_scale();
        let cam = calc_cam(scale);
        //returning the game object with the map and camera position
        Game {
            map,
            //setting the camera position to the center of the screen
            cam, //dyamic acc to the screen size
            scale,
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
        //update camera every frame so it always fits the screen
        //recalculate scale + cam every frame, so resizing always looks right
        self.scale = calc_scale();
        self.cam = calc_cam(self.scale);

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
            let (tx, ty) = to_tile(mx, my, self.cam, self.scale);

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
                        let (sx, sy) = to_screen(self.px, self.py, self.cam, self.scale);
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
                    let (sx, sy) = to_screen(self.px, self.py, self.cam, self.scale);

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
        let (sx, sy) = to_screen(
            self.monsters[idx].x,
            self.monsters[idx].y,
            self.cam,
            self.scale,
        );

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
                    draw_wall(x, y, self.cam, self.scale);
                } else {
                    let (sx, sy) = to_screen(x, y, self.cam, self.scale);
                    //draw a dot with circle shape
                    if self.gold.contains(&(x, y)) {
                        draw_circle(sx, sy + 16., 6. * self.scale, GOLD);
                    } else {
                        draw_circle(sx, sy + 16., 2. * self.scale, LIGHTGRAY);
                    }
                }
            }
        }

        //draw the path
        for (px, py) in &self.path {
            let (sx, sy) = to_screen(*px, *py, self.cam, self.scale);
            draw_circle(sx, sy + 16. * self.scale, 4. * self.scale, GOLD);
        }

        //draw the player character at the given x and y coordinates, using the camera position to adjust the screen coordinates
        draw_stickman(self.px, self.py, self.cam, self.scale, false);

        //draw monsters
        for m in &self.monsters {
            draw_stickman(m.x, m.y, self.cam, self.scale, true);
        }

        //draw floating text
        for t in &self.texts {
            if t.dmg < 0 {
                draw_text(&format!("+{}", -t.dmg), t.x, t.y, 20., GREEN);
            } else {
                draw_text(&format!("-{}", t.dmg), t.x, t.y, 20., RED);
            }
        }

        //HUD-(heads on display)- scaled font, safe distance from bottom
        let hud_size = (screen_width() * 0.05).clamp(16., 30.);
        let hud_y = screen_height() * 0.92; //92% down- always visible

        draw_text(&format!("HP: {}", self.hp), 12., hud_y, hud_size, BLACK);
        draw_text(
            &format!("SCORE: {}", self.score),
            12.,
            hud_y - hud_size - 4.,
            hud_size,
            BLACK,
        );
    }
}

/*==============MAIN======================== */
//title of the game window
#[macroquad::main("Crablo")]

async fn main() {
    let mut game = Game::new();
    let mut state = AppState::Menu;

    loop {
        clear_background(WHITE);

        // Scale menu font based on screen size
        let title_size = (screen_width() * 0.08).clamp(28., 60.);
        let sub_size = (screen_width() * 0.045).clamp(16., 32.);

        match state {
            /*==============MENU======================== */
            AppState::Menu => {
                let title = "CRABLO";
                let sub = "Click or tap to start";

                let title_w = title.len() as f32 * title_size * 0.6;
                let sub_w = sub.len() as f32 * sub_size * 0.5;

                draw_text(
                    title,
                    screen_width() / 2. - title_w / 2.,
                    screen_height() / 2. - title_size,
                    title_size,
                    BLACK,
                );

                draw_text(
                    sub,
                    screen_width() / 2. - sub_w / 2.,
                    screen_height() / 2. + sub_size,
                    sub_size,
                    GRAY,
                );

                //if the user presses enter, change the state to playing
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    game = Game::new();
                    state = AppState::Playing;
                }
            }

            /*==============PLAYING======================== */
            //if the state is playing, update the game and check if the game is over
            AppState::Playing => {
                if game.update(get_frame_time()) {
                    state = AppState::GameOver;
                }
                //draw the game state
                game.draw();
            }

            /*==============GAME OVER/VICTORY======================== */
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

                //----score msg & restart msg---
                let score_msg = format!("Final Score: {}", game.score);
                let restart_msg = "Tap or press enter to restart";

                let msg_w = msg.len() as f32 * title_size * 0.6;
                let score_w = score_msg.len() as f32 * sub_size * 0.5;
                let restart_w = restart_msg.len() as f32 * sub_size * 0.4;

                let cy = screen_height() / 2.;

                //center each line of text
                draw_text(
                    msg,
                    screen_width() / 2. - msg_w / 2.,
                    cy - title_size,
                    title_size,
                    col,
                );

                draw_text(
                    &score_msg,
                    screen_width() / 2. - score_w / 2.,
                    cy + sub_size,
                    sub_size,
                    BLACK,
                );

                //---restart----

                draw_text(
                    restart_msg,
                    screen_width() / 2. - restart_w / 2.,
                    cy + sub_size * 2.5,
                    sub_size * 0.85,
                    GRAY,
                );

                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    state = AppState::Menu
                }
            }
        }
        // wait for the next frame to be drawn
        next_frame().await;
    }
}
