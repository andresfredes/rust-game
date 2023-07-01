use std::cmp;
use tcod::colors::*;
use tcod::console::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const SCREEN_ORIGIN: (i32, i32) = (0, 0);

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOUR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100, };
const COLOUR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150,};

const OPAQUE: f32 = 1.0;
// const TRANSPARENT: f32 = 0.0;

const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    con: Offscreen,
}


#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    glyph: char,
    color: Color,
}


impl Object {
    pub fn new(x: i32, y: i32, glyph: char, color: Color) -> Self {
        Object { x: x, y: y, glyph: glyph, color: color }
    }
    
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.glyph, BackgroundFlag::None);
    }
}


#[derive(Clone, Copy,Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}


impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Rect{
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}


fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}


fn make_map() -> Map {
    let mut map = vec![
        vec![
            Tile::wall(); MAP_HEIGHT as usize
        ]; MAP_WIDTH as usize
    ];

    // Wall placement examples
    // map [30][22] = Tile::wall();
    // map [50][22] = Tile::wall();

    // Room placement examples
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    create_room(room1, &mut map);
    create_room(room2, &mut map);
    create_h_tunnel(25, 55, 23, &mut map);

    map
}


fn handle_keys(tcod: &mut Tcod, pc: &mut Object, game: &Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    match key {
        // Window
        Key { code: Enter, alt: true, .. } => {
            let is_fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!is_fullscreen);
        },
        Key { code: Escape, .. } => return true,

        // Movement
        Key { code: Up, .. } => pc.move_by(0, -1, game,),
        Key { code: Down , .. } => pc.move_by(0, 1, game,),
        Key { code: Left, .. } => pc.move_by(-1, 0, game,),
        Key { code: Right , ..} => pc.move_by(1, 0, game,),

        // Default (all other keys)
        _ => {}
    }
    
    false
}


fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // Set background
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con.set_char_background(
                    x, y, COLOUR_DARK_WALL, BackgroundFlag::Set
                );
            } else {
                tcod.con.set_char_background(
                    x, y, COLOUR_DARK_GROUND, BackgroundFlag::Set
                );
            }
        }
    }
    for object in objects {
        object.draw(&mut tcod.con);
    }

    // Add sub-consoles into root
    blit(
        &tcod.con,
        SCREEN_ORIGIN,
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        SCREEN_ORIGIN,
        OPAQUE,
        OPAQUE,
    );
}


fn main() {
    // Root console (window) properties
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust-game")
        .init();

    // Game-layer console properties
    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod { root, con };

    // FPS limit on loop, and therefore wait time (when waiting for user input)
    tcod::system::set_fps(LIMIT_FPS);

    // Object creation
    let pc = Object::new(25, 23, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', DARK_YELLOW);

    let mut objects = [pc, npc];
    let game = Game { map: make_map(), };

    // Game loop
    while !tcod.root.window_closed() {
        // Clear for new frame
        tcod.con.clear();

        // Draw all
        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();

        // Handle input
        let pc = &mut objects[0];
        let exit = handle_keys(&mut tcod, pc, &game);
        if exit {
            break;
        }
    }
}
