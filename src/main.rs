use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use rand::prelude::random;

const BOARD_WIDTH: i32 = 16;
const BOARD_HEIGHT: i32 = 16;
const UNREVEALED_TILE_COLOR: Color = Color::srgb(0.7, 0.0, 0.7);
const EMPTY_TILE_COLOR: Color = Color::srgb(0.0, 0.7, 0.7);
const BOMB_TILE_COLOR: Color = Color::srgb(0.7, 0.7, 0.0);
const MARKED_TILE_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BOMB_PROBABILITY: i32 = 20;

const INVALID_BOARD_INDEX: usize = usize::MAX;

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
struct RevealNeighborEvent {
    position: Position,
}

#[derive(Component, Debug)]
struct ShouldBeRevealed;

#[derive(Component, Debug)]
struct ShouldBeMarked;

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
        let x = id as i32 % BOARD_WIDTH;
        if x == 0 {
            y += 1;
        }
        let position = Position { x, y };

        // set the tile type randomly
        *tile_type = TileType::random();

        // spawn the tile
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: UNREVEALED_TILE_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Tile {
                adjacent_bomb_count: 0,
            })
            .insert(*tile_type)
            .insert(Size::square(0.9))
            .insert(position);
    }
}

fn board_idx(x: i32, y: i32) -> (usize, Position) {
    if x < 0 || x >= BOARD_WIDTH || y < 0 || y >= BOARD_HEIGHT {
        return (INVALID_BOARD_INDEX, Position { x: -1, y: -1 });
    }

    (((y * BOARD_WIDTH) + x) as usize, Position { x, y })
}

fn adjacent_idx_vec(x: i32, y: i32) -> Vec<(usize, Position)> {
    let mut vec: Vec<(usize, Position)> = Vec::new();

    vec.push(board_idx(x, y + 1));
    vec.push(board_idx(x + 1, y + 1));
    vec.push(board_idx(x + 1, y));
    vec.push(board_idx(x + 1, y - 1));
    vec.push(board_idx(x, y - 1));
    vec.push(board_idx(x - 1, y - 1));
    vec.push(board_idx(x - 1, y));
    vec.push(board_idx(x - 1, y + 1));

    let filtered_vec: Vec<(usize, Position)> = vec
        .into_iter()
        .filter(|(index, _)| *index != INVALID_BOARD_INDEX)
        .collect();
    filtered_vec
}

fn calculate_adjacent_bomb_counts(mut q: Query<(&mut Tile, &Position)>, board: Res<Board>) {
    for (mut tile, position) in q.iter_mut() {
        let mut adjacent_bomb_count = 0;
        let vec = adjacent_idx_vec(position.x, position.y);

        for (adjacent_idx, _) in vec.iter() {
            if board.tiles[*adjacent_idx] == TileType::Bomb {
                adjacent_bomb_count += 1;
            }
        }

        tile.adjacent_bomb_count = adjacent_bomb_count;
    }
}

fn handle_mouse_input(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    q: Query<(Entity, &Position)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    for mouse_button_event in mouse_button_input_events
        .read()
        .filter(|e| e.state == ButtonState::Released)
    {
        for cursor_moved_event in cursor_moved_events.read() {
            if let Ok(window) = windows.get_single() {
                let tile_size = window.width() / BOARD_WIDTH as f32;
                let mouse_event_position = cursor_moved_event.position;
                let mouse_position = Position {
                    x: ((mouse_event_position.x / tile_size) % window.width()) as i32,
                    y: (((mouse_event_position.y / tile_size) % window.height()) as i32
                        - BOARD_HEIGHT)
                        .abs()
                        - 1,
                };

                for (entity, tile_position) in q.iter() {
                    if mouse_position == *tile_position {
                        match mouse_button_event.button {
                            MouseButton::Left => {
                                commands.entity(entity).insert(ShouldBeRevealed {});
                            }
                            MouseButton::Right => {
                                commands.entity(entity).insert(ShouldBeMarked {});
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}

fn reveal_neighbor(
    mut commands: Commands,
    entity_position: Query<(Entity, &Position)>,
    mut reveal_neighbor_event_reader: EventReader<RevealNeighborEvent>,
) {
    for event in reveal_neighbor_event_reader.read() {
        let aiv = adjacent_idx_vec(event.position.x, event.position.y);
        let adjacent_positions: Vec<&Position> = aiv.iter().map(|(_, position)| position).collect();
        for (entity, _) in entity_position
            .iter()
            .filter(|(_, pos)| adjacent_positions.contains(pos))
        {
            commands.entity(entity).insert(ShouldBeRevealed {});
        }
    }
}

fn mark(mut commands: Commands, mut q: Query<(Entity, &mut Sprite, &ShouldBeMarked)>) {
    for (entity, mut sprite, _) in q.iter_mut() {
        sprite.color = MARKED_TILE_COLOR;

        commands.entity(entity).remove::<ShouldBeMarked>();
    }
}

fn reveal(
    mut commands: Commands,
    mut entities_to_be_revealed: Query<(
        Entity,
        &mut Sprite,
        &Tile,
        &TileType,
        &Position,
        &ShouldBeRevealed,
    )>,
    mut reveal_neighbor_event_writer: EventWriter<RevealNeighborEvent>,
) {
    let vec: Vec<(
        Entity,
        &Sprite,
        &Tile,
        &TileType,
        &Position,
        &ShouldBeRevealed,
    )> = entities_to_be_revealed.iter().collect();
    info!("Reveal length: {}", vec.len());
    for (entity, mut sprite, tile, tile_type, position, _) in entities_to_be_revealed.iter_mut() {
        match tile_type {
            TileType::Bomb => {
                sprite.color = BOMB_TILE_COLOR;
                info!("GAME OVER")
                // TODO handle game over state
            }
            TileType::Empty => {
                sprite.color = EMPTY_TILE_COLOR;

                if tile.adjacent_bomb_count == 0 {
                    reveal_neighbor_event_writer.send(RevealNeighborEvent {
                        position: *position,
                    });
                } else {
                    commands.entity(entity).with_children(|builder| {
                        builder.spawn(Text2dBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    format!("{}", tile.adjacent_bomb_count),
                                    TextStyle {
                                        font_size: 40.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                )],
                                justify: JustifyText::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                            },
                            transform: Transform {
                                scale: Vec3::new(0.01, 0.01, 1.),
                                ..default()
                            },
                            ..default()
                        });
                    });
                }
            }
        };

        commands.entity(entity).remove::<ShouldBeRevealed>();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minesweeper".into(),
                resolution: (BOARD_WIDTH as f32 * 50., BOARD_HEIGHT as f32 * 50.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Board::default())
        .add_systems(
            Startup,
            (
                setup_camera,
                fill_board,
                calculate_adjacent_bomb_counts,
                position_translation,
                size_scaling,
            )
                .chain(),
        )
        .add_systems(Update, (handle_mouse_input, reveal))
        .add_systems(PostUpdate, (mark, reveal_neighbor))
        .add_event::<RevealNeighborEvent>()
        .run();
}
