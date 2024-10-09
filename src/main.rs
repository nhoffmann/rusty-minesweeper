use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use rand::prelude::random;

// const BOARD_WIDTH: i32 = 9;
// const BOARD_HEIGHT: i32 = 9;

// const BOARD_WIDTH: i32 = 16;
// const BOARD_HEIGHT: i32 = 16;

const BOARD_WIDTH: i32 = 30;
const BOARD_HEIGHT: i32 = 16;

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
    pub board_width: i32,
    pub board_height: i32,
    pub tiles: Vec<TileType>,
}

impl Board {
    fn random_board(board_width: i32, board_height: i32, mine_count: u8) -> Self {
        let num_tiles: usize = (board_width * board_height) as usize;

        let mut board = Self {
            board_width,
            board_height,
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
        Self::random_board(16, 30, 99)
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
        if x < 0 || x >= self.board_width || y < 0 || y >= self.board_height {
            return (INVALID_BOARD_INDEX, Position { x: -1, y: -1 });
        }

        (((y * self.board_width) + x) as usize, Position { x, y })
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

fn size_scaling(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
    board: Res<Board>,
) {
    if let Ok(window) = windows.get_single() {
        for (sprite_size, mut transform) in q.iter_mut() {
            transform.scale = Vec3::new(
                sprite_size.width / board.board_width as f32 * window.width() as f32,
                sprite_size.height / board.board_height as f32 * window.height() as f32,
                1.0,
            );
        }
    }
}

fn position_translation(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
    board: Res<Board>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.0) + (tile_size / 2.0)
    }
    if let Ok(window) = windows.get_single() {
        for (pos, mut transform) in q.iter_mut() {
            transform.translation = Vec3::new(
                convert(
                    pos.x as f32,
                    window.width() as f32,
                    board.board_width as f32,
                ),
                convert(
                    pos.y as f32,
                    window.height() as f32,
                    board.board_height as f32,
                ),
                0.0,
            );
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup(mut commands: Commands, mut board: ResMut<Board>) {
    let mut y = -1;
    let board_width = board.board_width;
    for (id, tile_type) in board.tiles.iter_mut().enumerate() {
        let x = id as i32 % board_width;
        if x == 0 {
            y += 1;
        }
        let position = Position { x, y };

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
                revealed: false,
                adjacent_mine_count: 0,
            })
            .insert(*tile_type)
            .insert(Size::square(0.9))
            .insert(position);
    }
}

fn new_board(mut commands: Commands) {
    commands.insert_resource(Board::beginner());
}

fn deswpan_all(mut commands: Commands, entities: Query<Entity, With<Tile>>) {
    commands.remove_resource::<Board>();

    for entity in entities.iter() {
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
    windows: Query<&Window, With<PrimaryWindow>>,
    q: Query<(Entity, &Position)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    board: Res<Board>,
) {
    for mouse_button_event in mouse_button_input_events
        .read()
        .filter(|e| e.state == ButtonState::Released)
    {
        for cursor_moved_event in cursor_moved_events.read() {
            if let Ok(window) = windows.get_single() {
                let tile_size = window.width() / board.board_width as f32;
                let mouse_event_position = cursor_moved_event.position;
                let mouse_position = Position {
                    x: ((mouse_event_position.x / tile_size) % window.width()) as i32,
                    y: (((mouse_event_position.y / tile_size) % window.height()) as i32
                        - board.board_height)
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
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut sprite, mut tile, tile_type, position, _) in entities_to_be_revealed.iter_mut()
    {
        match tile_type {
            TileType::Mine => {
                sprite.color = BOMB_TILE_COLOR;
                game_state.set(GameState::Defeat);
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
    mut game_state: ResMut<NextState<GameState>>,
) {
    let marked_tiles_vec: Vec<&TileType> = marked_tiles
        .iter()
        .filter(|tt| **tt == TileType::Mine)
        .collect();

    if marked_tiles_vec.len() == board.mine_count as usize {
        game_state.set(GameState::Victory);
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

fn spawn_restart_button(mut commands: Commands) {
    let button_style = Style {
        width: Val::Px(250.0),
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

    commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ButtonAction::NewGame,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "New Game",
                button_text_style.clone(),
            ));
        });
}

fn handle_menu_buttons(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, Entity, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,

    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, entity, button_action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button_action {
                ButtonAction::NewGame => {
                    commands.entity(entity).despawn_recursive();
                    game_state.set(GameState::Playing);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Playing,
    Victory,
    Defeat,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum ButtonAction {
    NewGame,
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
        .init_state::<GameState>()
        .add_systems(
            OnEnter(GameState::Playing),
            (
                new_board,
                setup,
                calculate_adjacent_mine_counts,
                position_translation,
                size_scaling,
            )
                .chain(),
        )
        .add_systems(
            Startup,
            (
                setup_camera,
                setup,
                calculate_adjacent_mine_counts,
                position_translation,
                size_scaling,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                handle_mouse_input.run_if(in_state(GameState::Playing)),
                handle_menu_buttons.run_if(in_state(GameState::Defeat)),
                handle_menu_buttons.run_if(in_state(GameState::Victory)),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                mark,
                reveal,
                handle_reveal_neighbor_event,
                check_for_win.run_if(in_state(GameState::Playing)),
            )
                .chain(),
        )
        .add_systems(
            OnEnter(GameState::Defeat),
            (reveal_all, spawn_restart_button),
        )
        .add_systems(OnExit(GameState::Defeat), deswpan_all)
        .add_systems(
            OnEnter(GameState::Victory),
            (reveal_non_mine_tiles, spawn_restart_button),
        )
        .add_systems(OnExit(GameState::Victory), deswpan_all)
        .add_event::<RevealNeighborEvent>()
        .run();
}
