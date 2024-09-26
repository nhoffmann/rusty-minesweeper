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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
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
    // initially fill the board with empty tiles
    board.tiles = vec![TileType::Empty; num_tiles];

    let mut y = -1;
    for (id, tile_type) in board.tiles.iter_mut().enumerate() {
        let x = (id as f32 % BOARD_WIDTH as f32) as i32;
        if x == 0 {
            y += 1;
        }
        let position = Position { x, y };

        // set the tile type randomly
        *tile_type = TileType::random();

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
            .insert(*tile_type)
            .insert(Size::square(0.9))
            .insert(position);
    }
}

fn board_idx(x: i32, y: i32) -> i32 {
    ((y * BOARD_WIDTH) + x) as i32
}

fn calculate_adjacent_bomb_counts(mut q: Query<(&mut Tile, &Position)>, board: Res<Board>) {
    fn adjacent_idx_vec(x: i32, y: i32) -> Vec<i32> {
        let mut vec: Vec<i32> = vec![-1; 8];

        vec[0] = board_idx(x, y + 1);
        vec[1] = board_idx(x + 1, y + 1);
        vec[2] = board_idx(x + 1, y);
        vec[3] = board_idx(x + 1, y - 1);
        vec[4] = board_idx(x, y - 1);
        vec[5] = board_idx(x - 1, y - 1);
        vec[6] = board_idx(x - 1, y);
        vec[7] = board_idx(x - 1, y + 1);

        vec
    }

    for (mut tile, position) in q.iter_mut() {
        let mut adjacent_bomb_count = 0;
        let vec = adjacent_idx_vec(position.x, position.y);
        // info!("Adjacent idx vec: {:?}", vec);

        for adjacent_idx in vec {
            if adjacent_idx >= 0
                && adjacent_idx < board.tiles.len() as i32
                && board.tiles[adjacent_idx as usize] == TileType::Bomb
            {
                adjacent_bomb_count += 1;
            }
        }

        tile.adjacent_bomb_count = adjacent_bomb_count;
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
    mut commands: Commands,
    mut reveal_event_reader: EventReader<RevealEvent>,
    mut tiles: Query<Entity, With<Tile>>,
    mut q: Query<(Entity, &mut Sprite, &mut Tile, &Position, &TileType)>,
) {
    if let Some(reveal_event) = reveal_event_reader.read().next() {
        // let mut revealed_tile = Tile { revealed: false };

        for (entity, mut sprite, mut tile, position, tile_type) in q.iter_mut() {
            if position == &reveal_event.position {
                tile.revealed = true;
                match tile_type {
                    TileType::Bomb => {
                        sprite.color = BOMB_TILE_COLOR;
                        info!("GAME OVER")
                        // TODO handle game over state
                    }
                    TileType::Empty => {
                        sprite.color = EMPTY_TILE_COLOR;
                        info!("Adjacent bomb count: {}", tile.adjacent_bomb_count);
                        // revealed_tile = *tile;
                        // reveal the current tile
                        // -> show a number how many adjacent bomb tiles it has
                        let the_entity = tiles.get_mut(entity).unwrap();

                        commands.entity(the_entity).with_children(|builder| {
                            builder.spawn(Text2dBundle {
                                text: Text {
                                    sections: vec![TextSection::new(
                                        format!("{}", tile.adjacent_bomb_count),
                                        TextStyle {
                                            font_size: 2.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    )],
                                    justify: JustifyText::Center,
                                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                                },
                                transform: Transform::from_translation(Vec3::Z),
                                ..default()
                            });

                            // text: Text::from_section(
                            //     format!("{}", tile.adjacent_bomb_count),
                            //     TextStyle {
                            //         font_size: 1.5,
                            //         color: Color::WHITE,
                            //         ..default()
                            //     },
                            // )
                            // .with_justify(JustifyText::Center),
                            // ..default()
                        });

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
