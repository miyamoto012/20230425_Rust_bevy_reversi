use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_prototype_debug_lines::*;

pub const WINDOW_WIDTH: f32 = 500.0;
pub const WINDOW_HEIGHT: f32 = 500.0;

pub const GRID_X_LENGTH: u8 = 10;
pub const GRID_Y_LENGTH: u8 = 10;

pub const X_BOTTOM: f32 = -250.0;
pub const Y_BOTTOM: f32 = -250.0;

pub const COLOR_BLACK: Color = Color::rgb(0.85, 0.85, 0.85);
pub const COLOR_WHITE: Color = Color::rgb(0.15, 0.15, 0.15);
pub const COLOR_EMPTY: Color = Color::rgb(0.0, 0.5, 0.0);
pub const COLOR_WALL: Color = Color::rgb(0.5, 0.5, 0.5);

pub const SQUARE_SIZE: f32 = WINDOW_WIDTH / GRID_X_LENGTH as f32;

fn main() {
    App::new()
        .init_resource::<BoardState>()

        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "reversi!".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))

        .add_system(bevy::window::close_on_esc)

        .add_plugin(DebugLinesPlugin::default())
        .add_system(draw_grid)

        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_square)

        .add_system(input_click)

        .run();
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SquareState {
    Black,
    White,
    Empty,
    Wall,
}

impl SquareState {
    pub fn color(&self)->Color{
        match self {
            SquareState::Black => COLOR_BLACK,
            SquareState::White => COLOR_WHITE,
            SquareState::Empty => COLOR_EMPTY,
            SquareState::Wall => COLOR_WALL,
        }
    }

    pub fn invert_colors (&self) -> SquareState {
        match self {
            SquareState::Black => SquareState::White,
            SquareState::White => SquareState::Black,
            SquareState::Empty => SquareState::Empty,
            SquareState::Wall => SquareState::Wall,
        }
    }
}

pub const NONE_DIRECTION: u8 = 0b00000000;
pub const UPPER: u8 = 0b00000001;
pub const UPPER_LEFT: u8 = 0b00000010;
pub const LEFT: u8 = 0b00000100;
pub const LOWER_LEFT: u8 = 0b0000100;
pub const LOWER: u8 = 0b00010000;
pub const LOWER_RIGHT: u8 = 0b00100000;
pub const RIGHT: u8 = 0b01000000;
pub const UPPER_RIGHT: u8 = 0b10000000;

#[derive(Resource)]
pub struct BoardState {
    value : [[SquareState; GRID_X_LENGTH as usize]; GRID_Y_LENGTH as usize],
}

impl Default for BoardState {
    fn default() -> Self {
        let mut state = [
            [SquareState::Wall; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Empty; GRID_X_LENGTH as usize],
            [SquareState::Wall; GRID_X_LENGTH as usize],
        ];

        state[4][4] = SquareState::Black;
        state[5][5] = SquareState::Black;
        state[4][5] = SquareState::White;
        state[5][4] = SquareState::White;

        for i_y in 1..=8 {
            println!("i_y: {}", i_y);
            state[i_y][0] = SquareState::Wall;
            state[i_y][9] = SquareState::Wall;
        }

        BoardState {
            value : state
        }
    }
}

impl BoardState {
    fn check_mobility(&self, y: usize, x: usize, piece: SquareState) -> u8{

        let selected_square = self.value[y][x];

        if selected_square != SquareState::Empty {
            return NONE_DIRECTION;
        }

        if piece != SquareState::Black && piece != SquareState::White {
            panic!();
        }

        let mut direction = NONE_DIRECTION;

        //マスの上をチェック
        if self.value[y+1][x] == piece.invert_colors() {
            let mut i_y = y + 2;
            while self.value[i_y][x] == piece.invert_colors() {
                i_y += 1;
            }
            if self.value[i_y][x] == piece {
                direction |= UPPER;
            }
        }

        //マスの右上をチェック
        if self.value[y+1][x+1] == piece.invert_colors() {
            let mut i_y = y + 2;
            let mut i_x = x + 2;
            while self.value[i_y][i_x] == piece.invert_colors() {
                i_y += 1;
                i_x += 1;
            }
            if self.value[i_y][i_x] == piece {
                direction |= UPPER_RIGHT;
            }
        }

        //マスの右をチェック
        if self.value[y][x+1] == piece.invert_colors() {
            let mut i_x = x + 2;
            while self.value[y][i_x] == piece.invert_colors() {
                i_x += 1;
            }
            if self.value[y][i_x] == piece {
                direction |= RIGHT;
            }
        }

        //マスの右下をチェック
        if self.value[y-1][x+1] == piece.invert_colors() {
            let mut i_y = y - 2;
            let mut i_x = x + 2;
            while self.value[i_y][i_x] == piece.invert_colors() {
                i_y -= 1;
                i_x += 1;
            }
            if self.value[i_y][i_x] == piece {
                direction |= LOWER_RIGHT;
            }
        }

        //マスの下をチェック
        if self.value[y-1][x] == piece.invert_colors() {
            let mut i_y = y - 2;
            while self.value[i_y][x] == piece.invert_colors() {
                i_y -= 1;
            }
            if self.value[i_y][x] == piece {
                direction |= LOWER;
            }
        }

        //マスの左下をチェック
        if self.value[y-1][x-1] == piece.invert_colors() {
            let mut i_y = y - 2;
            let mut i_x = x - 2;
            while self.value[i_y][i_x] == piece.invert_colors() {
                i_y -= 1;
                i_x -= 1;
            }
            if self.value[i_y][i_x] == piece {
                direction |= LOWER_LEFT;
            }
        }

        //マスの左をチェック
        if self.value[y][x-1] == piece.invert_colors() {
            let mut i_x = x - 2;
            while self.value[y][i_x] == piece.invert_colors() {
                i_x -= 1;
            }
            if self.value[y][i_x] == piece {
                direction |= LEFT;
            }
        }

        //マスの左上をチェック
        if self.value[y+1][x-1] == piece.invert_colors() {
            let mut i_y = y + 2;
            let mut i_x = x - 2;
            while self.value[i_y][i_x] == piece.invert_colors() {
                i_y += 1;
                i_x -= 1;
            }
            if self.value[i_y][i_x] == piece {
                direction |= UPPER_LEFT;
            }
        }

        return direction;
    }
}

#[derive(Component, Copy, Clone)]
pub struct Position {
    x: u8,
    y: u8,
}

pub fn spawn_camera(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
}

pub fn spawn_square(
    mut commands: Commands,
    board_state: Res<BoardState>
){
    for i_y in 0..GRID_Y_LENGTH {
        for i_x in 0..GRID_X_LENGTH {
            let positon = Position { x: i_x, y: i_y };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: board_state.value[i_y as usize][i_x as usize].color(),
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 10.0),
                        translation: position_translation(positon),
                        ..default()
                    },
                    ..default()
                },
                Position {
                    x: positon.x,
                    y: positon.y,
                },
            ));
        }
    }

    fn position_translation(position: Position)->Vec3{
        let x: f32 = position.x as f32 * SQUARE_SIZE + SQUARE_SIZE/2.0 + X_BOTTOM;
        let y: f32 = (GRID_Y_LENGTH - position.y - 1) as f32 * SQUARE_SIZE + SQUARE_SIZE/2.0 + Y_BOTTOM;

        return Vec3::new(x, y, 0.0);
    }
}

pub fn draw_grid(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut lines: ResMut<DebugLines>
) {
    let window = window_query.get_single().unwrap();
    let half_win_width = 0.5 * window.width();
    let half_win_height = 0.5 * window.height();
    let x_space = window.width() / GRID_X_LENGTH as f32;
    let y_space = window.height() / GRID_Y_LENGTH as f32;

    let mut i = -1. * half_win_height;
    while i < half_win_height {
        lines.line(
            Vec3::new(-1. * half_win_width, i, 0.0),
            Vec3::new(half_win_width, i, 0.0),
            0.0,
        );
        i += y_space;
    }

    i = -1. * half_win_width;
    while i < half_win_width {
        lines.line(
            Vec3::new(i, -1. * half_win_height, 0.0),
            Vec3::new(i, half_win_height, 0.0),
            0.0,
        );
        i += x_space;
    }

    lines.line(
        Vec3::new(0., -1. * half_win_height, 0.0),
        Vec3::new(0., half_win_height, 0.0),
        0.0,
    );
}

pub fn input_click (
    board_state: ResMut<BoardState>,
    mouse_button: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
){
    let window = window_query.get_single().unwrap();

    if ! mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(position) = window.cursor_position() {
        let x = (position.x / SQUARE_SIZE) as usize;
        let y = (position.y / SQUARE_SIZE) as usize;

        println!("position.x: {}", x );
        println!("position.y: {}", y );
        println!("state: {:?}", board_state.value[y][x]);
        println!("stararer: {:08b}", board_state.check_mobility(y, x, SquareState::White));
    }
}