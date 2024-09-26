use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use rand::prelude::random;

const BOARD_WIDTH: i32 = 6;
const BOARD_HEIGHT: i32 = 4;
const UNREVEALED_TILE_COLOR: Color = Color::srgb(0.7, 0.0, 0.7);
const EMPTY_TILE_COLOR: Color = Color::srgb(0.0, 0.7, 0.7);
const BOMB_TILE_COLOR: Color = Color::srgb(0.7, 0.7, 0.0);
const BOMB_PROBABILITY: i32 = 20;

#[derive(Resource, Default)]
struct Board {
    pub tiles: Vec<TileType>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn adjacent(&self, other: &Position) -> bool {
        (self.x - other.x).abs() <= 2 && (self.y - other.y).abs() <= 2
    }
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Tile {
    revealed: bool,
    adjacent_bomb_count: u8,
}

#[derive(Component, Clone, Copy, Debug)]
pub enum TileType {
    Bomb,
    Empty,
}

impl TileType {
    pub fn random() -> Self {
        let random_number = (random::<f32>() * 100.) as i32;

        if random_number < BOMB_PROBABILITY {
            TileType::Bomb
        } else {
            TileType::Empty
        }
    }
}

#[derive(Event, Debug)]
struct RevealEvent {
    position: Position,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn size_scaling(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    if let Ok(window) = windows.get_single() {
        for (sprite_size, mut transform) in q.iter_mut() {
            transform.scale = Vec3::new(
                sprite_size.width / BOARD_WIDTH as f32 * window.width() as f32,
                sprite_size.height / BOARD_HEIGHT as f32 * window.height() as f32,
                1.0,
            );
        }
    }
}

fn position_translation(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.0) + (tile_size / 2.0)
    }
    if let Ok(window) = windows.get_single() {
        for (pos, mut transform) in q.iter_mut() {
            transform.translation = Vec3::new(
                convert(pos.x as f32, window.width() as f32, BOARD_WIDTH as f32),
                convert(pos.y as f32, window.height() as f32, BOARD_HEIGHT as f32),
                0.0,
            );
        }
    }
}

fn fill_board(mut commands: Commands, mut board: ResMut<Board>) {
    let num_tiles: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;
    board.tiles = vec![TileType::Empty; num_tiles];

    let mut y = -1;
    for (id, _tile_type) in board.tiles.iter_mut().enumerate() {
        let x = (id as f32 % BOARD_WIDTH as f32) as i32;
        if x == 0 {
            y += 1;
        }
        let position = Position { x, y };

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: UNREVEALED_TILE_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Tile {
                revealed: false,
                adjacent_bomb_count: 0,
            })
            .insert(TileType::random())
            .insert(Size::square(0.9))
            .insert(position);
    }
}

fn board_idx(x: i32, y: i32) -> usize {
    ((y * BOARD_WIDTH) + x) as usize
}

fn calculate_adjacent_bomb_counts(mut commands: Commands, mut q: Query<(&mut Tile, &Position)>) {
    for (tile, position) in q.iter_mut() {
        info!(
            "Board index: {} | position: {:?}",
            board_idx(position.x, position.y),
            position
        );
    }
}

fn handle_mouse_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    tile_positions: Query<(&Position, &Tile)>,
    mut reveal_event_writer: EventWriter<RevealEvent>,
) {
    for _ in mouse_button_input_events
        .read()
        .filter(|e| e.state == ButtonState::Released)
    {
        for event in cursor_moved_events.read() {
            if let Ok(window) = windows.get_single() {
                let tile_size = window.width() / BOARD_WIDTH as f32;
                let mouse_event_position = event.position;
                let mouse_position = Position {
                    x: ((mouse_event_position.x / tile_size) % window.width()) as i32,
                    y: (((mouse_event_position.y / tile_size) % window.height()) as i32
                        - BOARD_HEIGHT)
                        .abs()
                        - 1,
                };

                for (tile_position, tile) in tile_positions.iter() {
                    if mouse_position.x == tile_position.x
                        && mouse_position.y == tile_position.y
                        && !tile.revealed
                    {
                        reveal_event_writer.send(RevealEvent {
                            position: tile_position.clone(),
                        });
                    }
                }
            }
        }
    }
}

fn reveal(
    mut reveal_event_reader: EventReader<RevealEvent>,
    mut q: Query<(&mut Sprite, &mut Tile, &Position, &TileType)>,
) {
    if let Some(reveal_event) = reveal_event_reader.read().next() {
        // let mut revealed_tile = Tile { revealed: false };

        for (mut sprite, mut tile, position, tile_type) in q.iter_mut() {
            if position == &reveal_event.position {
                tile.revealed = true;
                match tile_type {
                    TileType::Bomb => {
                        sprite.color = BOMB_TILE_COLOR;
                        // GAME OVER
                    }
                    TileType::Empty => {
                        sprite.color = EMPTY_TILE_COLOR;
                        // revealed_tile = *tile;
                        // reveal the current tile
                        // -> show a number how many adjacent bomb tiles it has
                        // or
                        // -> if not adjacent to bomb tile, reveal all adjacent not adjacent to bomb tiles
                    }
                };
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minesweeper".into(),
                resolution: (600., 400.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Board::default())
        .add_systems(
            Startup,
            (setup_camera, fill_board, calculate_adjacent_bomb_counts).chain(),
        )
        .add_systems(Update, handle_mouse_input)
        .add_systems(
            PostUpdate,
            ((position_translation, size_scaling).chain(), reveal),
        )
        .add_event::<RevealEvent>()
        .run();
}
