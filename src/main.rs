use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use rand::prelude::random;

const WINDOW_WIDTH: i32 = 30;
const WINDOW_HEIGHT: i32 = 16;

const TILE_SIZE: f32 = 50.;
const FIELD_SIZE: f32 = 0.9;

const UNREVEALED_TILE_COLOR: Color = Color::srgb(0.7, 0.0, 0.7);
const EMPTY_TILE_COLOR: Color = Color::srgb(0.0, 0.7, 0.7);
const BOMB_TILE_COLOR: Color = Color::srgb(0.7, 0.7, 0.0);
const MARKED_TILE_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const INVALID_BOARD_INDEX: usize = usize::MAX;

#[derive(Resource)]
struct Board {
    pub mine_count: u8,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileType>,
}

impl Board {
    fn random_board(board_width: i32, board_height: i32, mine_count: u8) -> Self {
        let num_tiles: usize = (board_width * board_height) as usize;

        let mut board = Self {
            width: board_width,
            height: board_height,
            mine_count,
            tiles: vec![TileType::Empty; num_tiles],
        };

        while board.count_mine_tiles() < board.mine_count as i32 {
            let random_index = random::<f32>() * board.tiles.len() as f32;
            board.tiles[random_index as usize] = TileType::Mine;
        }

        board
    }

    fn beginner() -> Self {
        Self::random_board(9, 9, 10)
    }

    fn intermediate() -> Self {
        Self::random_board(16, 16, 40)
    }

    fn expert() -> Self {
        Self::random_board(30, 16, 99)
    }

    fn count_mine_tiles(&self) -> i32 {
        let mine_tiles_vec: Vec<&TileType> = self
            .tiles
            .iter()
            .filter(|tt| **tt == TileType::Mine)
            .collect();
        mine_tiles_vec.len() as i32
    }

    fn board_idx(&self, x: i32, y: i32) -> (usize, Position) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return (INVALID_BOARD_INDEX, Position { x: -1, y: -1 });
        }

        (((y * self.width) + x) as usize, Position { x, y })
    }

    fn adjacent_idx_vec(&self, x: i32, y: i32) -> Vec<(usize, Position)> {
        let mut vec: Vec<(usize, Position)> = Vec::new();

        // Top
        vec.push(self.board_idx(x, y + 1));
        // Top-right
        vec.push(self.board_idx(x + 1, y + 1));
        // Right
        vec.push(self.board_idx(x + 1, y));
        // Bottom-right
        vec.push(self.board_idx(x + 1, y - 1));
        //Bottom
        vec.push(self.board_idx(x, y - 1));
        // Bottom-left
        vec.push(self.board_idx(x - 1, y - 1));
        // Left
        vec.push(self.board_idx(x - 1, y));
        // Top-left
        vec.push(self.board_idx(x - 1, y + 1));

        // remove invalid board positions
        vec.into_iter()
            .filter(|(index, _)| *index != INVALID_BOARD_INDEX)
            .collect::<Vec<(usize, Position)>>()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
struct Tile {
    revealed: bool,
    adjacent_mine_count: u8,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Mine,
    Empty,
}

#[derive(Event, Debug)]
struct RevealNeighborEvent {
    position: Position,
}

#[derive(Component, Debug)]
struct ShouldBeRevealed;

#[derive(Component, Debug)]
struct ShouldBeMarked;

#[derive(Component, Debug)]
struct Marked;

#[derive(Component, Debug)]
struct Menu;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup(mut commands: Commands, board: Res<Board>) {
    let offset_x: f32 = board.width as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let offset_y: f32 = board.height as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;

    let mut y = -1;
    for (id, tile_type) in board.tiles.iter().enumerate() {
        let x = id as i32 % board.width;
        if x == 0 {
            y += 1;
        }

        // spawn the tile
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: UNREVEALED_TILE_COLOR,
                    ..default()
                },
                transform: Transform {
                    scale: Vec3::new(FIELD_SIZE * TILE_SIZE, FIELD_SIZE * TILE_SIZE, 1.0),
                    translation: Vec3::new(
                        x as f32 * TILE_SIZE - offset_x,
                        y as f32 * TILE_SIZE - offset_y,
                        0.0,
                    ),
                    ..default()
                },
                ..default()
            },
            Tile::default(),
            Position { x, y },
            *tile_type,
        ));
    }
}

fn despawn_all(
    mut commands: Commands,
    tiles: Query<Entity, With<Tile>>,
    menu: Query<Entity, With<Menu>>,
) {
    for entity in tiles.iter().chain(menu.iter()) {
        commands.entity(entity).despawn_recursive();
    }
}

fn calculate_adjacent_mine_counts(mut q: Query<(&mut Tile, &Position)>, board: Res<Board>) {
    for (mut tile, position) in q.iter_mut() {
        let mut adjacent_mine_count = 0;
        let vec = board.adjacent_idx_vec(position.x, position.y);

        for (adjacent_idx, _) in vec.iter() {
            if board.tiles[*adjacent_idx] == TileType::Mine {
                adjacent_mine_count += 1;
            }
        }

        tile.adjacent_mine_count = adjacent_mine_count;
    }
}

fn handle_mouse_input(
    mut commands: Commands,
    q: Query<(Entity, &Transform)>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    fn in_bounds(pos: Vec2, x: f32, y: f32, length: f32) -> bool {
        pos.x >= x && pos.x <= (x + length) && pos.y >= y && pos.y <= (y + length)
    }

    let tile_offset = Vec2::new(TILE_SIZE / 2., TILE_SIZE / 2.);
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        for mouse_button_event in mouse_button_input_events
            .read()
            .filter(|e| e.state == ButtonState::Released)
        {
            for (entity, tile_position) in q.iter() {
                if in_bounds(
                    world_position + tile_offset,
                    tile_position.translation.x,
                    tile_position.translation.y,
                    TILE_SIZE,
                ) {
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

fn handle_reveal_neighbor_event(
    mut commands: Commands,
    entity_position: Query<(Entity, &Position)>,
    mut reveal_neighbor_event_reader: EventReader<RevealNeighborEvent>,
    board: Res<Board>,
) {
    for event in reveal_neighbor_event_reader.read() {
        let aiv = board.adjacent_idx_vec(event.position.x, event.position.y);
        let adjacent_positions: Vec<&Position> = aiv.iter().map(|(_, position)| position).collect();

        for (entity, _) in entity_position
            .iter()
            .filter(|(_, pos)| adjacent_positions.contains(pos))
        {
            commands.entity(entity).insert(ShouldBeRevealed {});
        }
    }
}

fn mark(mut commands: Commands, mut q: Query<(Entity, &mut Sprite, &Tile, &ShouldBeMarked)>) {
    for (entity, mut sprite, tile, _) in q.iter_mut() {
        if tile.revealed {
            continue;
        }

        sprite.color = MARKED_TILE_COLOR;

        commands
            .entity(entity)
            .remove::<ShouldBeMarked>()
            .insert(Marked {});
    }
}

fn reveal(
    mut commands: Commands,
    mut entities_to_be_revealed: Query<(
        Entity,
        &mut Sprite,
        &mut Tile,
        &TileType,
        &Position,
        &ShouldBeRevealed,
    )>,
    mut reveal_neighbor_event_writer: EventWriter<RevealNeighborEvent>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut sprite, mut tile, tile_type, position, _) in entities_to_be_revealed.iter_mut()
    {
        match tile_type {
            TileType::Mine => {
                sprite.color = BOMB_TILE_COLOR;
                next_game_state.set(GameState::Defeat);
            }
            TileType::Empty => {
                sprite.color = EMPTY_TILE_COLOR;

                if !tile.revealed && tile.adjacent_mine_count == 0 {
                    reveal_neighbor_event_writer.send(RevealNeighborEvent {
                        position: *position,
                    });
                } else if !tile.revealed {
                    commands.entity(entity).with_children(|builder| {
                        builder.spawn(Text2dBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    format!("{}", tile.adjacent_mine_count),
                                    TextStyle {
                                        font_size: 60.0,
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

        tile.revealed = true;
        commands.entity(entity).remove::<ShouldBeRevealed>();
    }
}

fn check_for_win(
    marked_tiles: Query<&TileType, With<Marked>>,
    board: Res<Board>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let marked_tiles_vec: Vec<&TileType> = marked_tiles
        .iter()
        .filter(|tt| **tt == TileType::Mine)
        .collect();

    if marked_tiles_vec.len() == board.mine_count as usize {
        next_game_state.set(GameState::Victory);
    }
}

fn reveal_all(mut commands: Commands, entities: Query<Entity, With<Tile>>) {
    for entity in entities.iter() {
        commands.entity(entity).insert(ShouldBeRevealed);
    }
}

fn reveal_non_mine_tiles(mut commands: Commands, entities: Query<(Entity, &TileType)>) {
    for (entity, _) in entities.iter().filter(|(_, tt)| **tt == TileType::Empty) {
        commands.entity(entity).insert(ShouldBeRevealed {});
    }
}

fn spawn_menu(mut commands: Commands) {
    let button_style = Style {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    // Main container for the main_menu interface
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    background_color: BackgroundColor(TEXT_COLOR),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            ButtonAction::BeginnerGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Beginner",
                                button_text_style.clone(),
                            ));
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            ButtonAction::IntermediateGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Intermediate",
                                button_text_style.clone(),
                            ));
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            ButtonAction::ExpertGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Expert",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}

fn handle_menu_buttons(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button_action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button_action {
                ButtonAction::BeginnerGame => {
                    commands.insert_resource(Board::beginner());
                }
                ButtonAction::IntermediateGame => {
                    commands.insert_resource(Board::intermediate());
                }
                ButtonAction::ExpertGame => {
                    commands.insert_resource(Board::expert());
                }
            }

            next_game_state.set(GameState::Playing);
        }
    }
}

fn transition_to_menu(mut next_game_state: ResMut<NextState<GameState>>) {
    next_game_state.set(GameState::Menu);
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Victory,
    Defeat,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum ButtonAction {
    BeginnerGame,
    IntermediateGame,
    ExpertGame,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Minesweeper".into(),
                    canvas: Some("#game-canvas".into()),
                    resolution: (
                        WINDOW_WIDTH as f32 * TILE_SIZE,
                        WINDOW_HEIGHT as f32 * TILE_SIZE,
                    )
                        .into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .init_state::<GameState>()
        .add_systems(
            OnEnter(GameState::Playing),
            (setup, calculate_adjacent_mine_counts).chain(),
        )
        .add_systems(Startup, (setup_camera, spawn_menu).chain())
        .add_systems(
            Update,
            (
                handle_mouse_input.run_if(in_state(GameState::Playing)),
                handle_menu_buttons.run_if(in_state(GameState::Menu)),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                reveal,
                (mark, handle_reveal_neighbor_event, check_for_win)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
                transition_to_menu.run_if(in_state(GameState::Defeat)),
                transition_to_menu.run_if(in_state(GameState::Victory)),
            ),
        )
        .add_systems(OnEnter(GameState::Defeat), (reveal_all, spawn_menu))
        .add_systems(
            OnEnter(GameState::Victory),
            (reveal_non_mine_tiles, spawn_menu),
        )
        .add_systems(OnExit(GameState::Menu), despawn_all)
        .add_event::<RevealNeighborEvent>()
        .run();
}
