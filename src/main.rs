use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_prototype_debug_lines::*;

pub const WINDOW_WIDTH: f32 = 500.0;
pub const WINDOW_HEIGHT: f32 = 500.0;

pub const GRID_X_LENGTH: u8 = 10;
pub const GRID_Y_LENGTH: u8 = 10;

pub const X_BOTTOM: f32 = -250.0;
pub const Y_BOTTOM: f32 = -250.0;

pub const COLOR_BLACK: Color = Color::rgb(0.15, 0.15, 0.15);
pub const COLOR_WHITE: Color = Color::rgb(0.85, 0.85, 0.85);
pub const COLOR_EMPTY: Color = Color::rgb(0.0, 0.5, 0.0);
pub const COLOR_WALL: Color = Color::rgb(0.5, 0.5, 0.5);

pub const SQUARE_SIZE: f32 = WINDOW_WIDTH / GRID_X_LENGTH as f32;

pub const NONE_DIRECTION: u8 = 0b00000000;
pub const UPPER: u8 = 0b00000001;
pub const UPPER_LEFT: u8 = 0b00000010;
pub const LEFT: u8 = 0b00000100;
pub const LOWER_LEFT: u8 = 0b0001000;
pub const LOWER: u8 = 0b00010000;
pub const LOWER_RIGHT: u8 = 0b00100000;
pub const RIGHT: u8 = 0b01000000;
pub const UPPER_RIGHT: u8 = 0b10000000;

fn main() {
    App::new()
        .init_resource::<Board>()
        .init_resource::<Turns>()
        .init_resource::<UpdateLog>()
        .add_state::<TurnColorState>()
        .add_event::<FlipEvent>()

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

        .add_system(check_click)
        .add_system(flip_colors)
        .add_system(update_board_display)
        .add_system(undo)

        .run();
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Square {
    Black,
    White,
    Empty,
    Wall,
}

impl Square {
    pub fn color(&self)->Color{
        match self {
            Square::Black => COLOR_BLACK,
            Square::White => COLOR_WHITE,
            Square::Empty => COLOR_EMPTY,
            Square::Wall => COLOR_WALL,
        }
    }

    pub fn invert_colors (&self) -> Square {
        match self {
            Square::Black => Square::White,
            Square::White => Square::Black,
            Square::Empty => Square::Empty,
            Square::Wall => Square::Wall,
        }
    }
}

#[derive(Resource)]
pub struct Board {
    squares : [[Square; GRID_X_LENGTH as usize]; GRID_Y_LENGTH as usize],
}

impl Default for Board {
    fn default() -> Self {
        let mut state = [
            [Square::Wall; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Empty; GRID_X_LENGTH as usize],
            [Square::Wall; GRID_X_LENGTH as usize],
        ];

        state[4][4] = Square::Black;
        state[5][5] = Square::Black;
        state[4][5] = Square::White;
        state[5][4] = Square::White;

        for i_y in 1..=8 {
            state[i_y][0] = Square::Wall;
            state[i_y][9] = Square::Wall;
        }

        Board {
            squares : state
        }
    }
}

impl Board {
    pub fn check_mobility(&self, y: usize, x: usize, piece: Square) -> u8{

        let selected_square = self.squares[y][x];

        if selected_square != Square::Empty {
            return NONE_DIRECTION;
        }

        if piece != Square::Black && piece != Square::White {
            panic!();
        }

        let mut direction = NONE_DIRECTION;

        //マスの上をチェック
        if self.squares[y+1][x] == piece.invert_colors() {
            let mut i_y = y + 2;
            while self.squares[i_y][x] == piece.invert_colors() {
                i_y += 1;
            }
            if self.squares[i_y][x] == piece {
                direction |= UPPER;
            }
        }

        //マスの右上をチェック
        if self.squares[y+1][x+1] == piece.invert_colors() {
            let mut i_y = y + 2;
            let mut i_x = x + 2;
            while self.squares[i_y][i_x] == piece.invert_colors() {
                i_y += 1;
                i_x += 1;
            }
            if self.squares[i_y][i_x] == piece {
                direction |= UPPER_RIGHT;
            }
        }

        //マスの右をチェック
        if self.squares[y][x+1] == piece.invert_colors() {
            let mut i_x = x + 2;
            while self.squares[y][i_x] == piece.invert_colors() {
                i_x += 1;
            }
            if self.squares[y][i_x] == piece {
                direction |= RIGHT;
            }
        }

        //マスの右下をチェック
        if self.squares[y-1][x+1] == piece.invert_colors() {
            let mut i_y = y - 2;
            let mut i_x = x + 2;
            while self.squares[i_y][i_x] == piece.invert_colors() {
                i_y -= 1;
                i_x += 1;
            }
            if self.squares[i_y][i_x] == piece {
                direction |= LOWER_RIGHT;
            }
        }

        //マスの下をチェック
        if self.squares[y-1][x] == piece.invert_colors() {
            let mut i_y = y - 2;
            while self.squares[i_y][x] == piece.invert_colors() {
                i_y -= 1;
            }
            if self.squares[i_y][x] == piece {
                direction |= LOWER;
            }
        }

        //マスの左下をチェック
        if self.squares[y-1][x-1] == piece.invert_colors() {
            let mut i_y = y - 2;
            let mut i_x = x - 2;
            while self.squares[i_y][i_x] == piece.invert_colors() {
                i_y -= 1;
                i_x -= 1;
            }
            if self.squares[i_y][i_x] == piece {
                direction |= LOWER_LEFT;
            }
        }

        //マスの左をチェック
        if self.squares[y][x-1] == piece.invert_colors() {
            let mut i_x = x - 2;
            while self.squares[y][i_x] == piece.invert_colors() {
                i_x -= 1;
            }
            if self.squares[y][i_x] == piece {
                direction |= LEFT;
            }
        }

        //マスの左上をチェック
        if self.squares[y+1][x-1] == piece.invert_colors() {
            let mut i_y = y + 2;
            let mut i_x = x - 2;
            while self.squares[i_y][i_x] == piece.invert_colors() {
                i_y += 1;
                i_x -= 1;
            }
            if self.squares[i_y][i_x] == piece {
                direction |= UPPER_LEFT;
            }
        }

        return direction;
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum TurnColorState {
    #[default]
    Black,
    White,
}

#[derive(Resource)]
pub struct Turns {
    count: u8
}

impl Default for Turns {
    fn default() -> Self {
        Turns {
            count: 0
        }
    }
}

#[derive(Resource, Debug)]
pub struct UpdateLog {
    put: Vec<Position>,
    flip: Vec<Vec<Position>>
}

impl Default for UpdateLog {
    fn default() -> Self {
        UpdateLog{
            put: Vec::new(),
            flip: Vec::new(),
        }
    }
}

impl TurnColorState {
    pub fn put_square (&self) -> Square {
        match self {
            TurnColorState::Black => Square::Black,
            TurnColorState::White => Square::White,
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Position {
    x: u8,
    y: u8,
}

pub struct FlipEvent {
    x: usize,
    y: usize,
    direction: u8,
}

pub fn spawn_camera(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
}

pub fn spawn_square(
    mut commands: Commands,
    board: Res<Board>,
){
    for i_y in 0..GRID_Y_LENGTH {
        for i_x in 0..GRID_X_LENGTH {
            let positon = Position { x: i_x, y: i_y };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: board.squares[i_y as usize][i_x as usize].color(),
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
        let y: f32 = position.y as f32 * SQUARE_SIZE + SQUARE_SIZE/2.0 + Y_BOTTOM;

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

pub fn update_board_display (
    board: Res<Board>,
    mut square_query: Query<(&mut Sprite, &Position)>,
){
    for (mut sprite, position) in square_query.iter_mut() {
        let x = position.x as usize;
        let y = position.y as usize;

        sprite.color = board.squares[y][x].color();
    }
}

pub fn check_click (
    board: Res<Board>,
    mouse_button: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut flip_event_writer: EventWriter<FlipEvent>,
    turn_color_state: Res<State<TurnColorState>>,
){
    let window = window_query.get_single().unwrap();

    if ! mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(position) = window.cursor_position() {
        let x = (position.x / SQUARE_SIZE) as usize;
        let y = (position.y / SQUARE_SIZE) as usize;

        let direction = board.check_mobility(
            y, x, turn_color_state.0.put_square());

        if direction != NONE_DIRECTION {
            flip_event_writer.send(FlipEvent { x: x, y: y, direction: direction})
        }
    }
}

pub fn flip_colors (
    mut board: ResMut<Board>,
    mut flip_event_reader: EventReader<FlipEvent>,
    mut turn_color_state: ResMut<State<TurnColorState>>,
    mut update_log: ResMut<UpdateLog>,
    mut turns: ResMut<Turns>,
){
    for flip_event in flip_event_reader.iter() {
        let x = flip_event.x;
        let y = flip_event.y;
        let direction = flip_event.direction;

        let put_square = turn_color_state.0.put_square();
        board.squares[y][x] = put_square;

        update_log.put.push(Position { x: x as u8, y: y as u8 });
        update_log.flip.push(Vec::new());

        if (direction & UPPER) != NONE_DIRECTION {
            let mut i_y = y + 1;

            while board.squares[i_y][x] != put_square {
                board.squares[i_y][x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: x as u8, y: i_y as u8 });

                i_y += 1;
            }

        }

        if (direction & UPPER_RIGHT) != NONE_DIRECTION {
            let mut i_y = y + 1;
            let mut i_x = x + 1;

            while board.squares[i_y][i_x] != put_square {
                board.squares[i_y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: i_y as u8 });

                i_y += 1;
                i_x += 1;
            }

        }

        if (direction & RIGHT) != NONE_DIRECTION {
            let mut i_x = x + 1;

            while board.squares[y][i_x] != put_square {
                board.squares[y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: y as u8 });

                i_x += 1;
            }

        }

        if (direction & LOWER_RIGHT) != NONE_DIRECTION {
            let mut i_y = y - 1;
            let mut i_x = x + 1;

            while board.squares[i_y][i_x] != put_square {
                board.squares[i_y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: i_y as u8 });

                i_y -= 1;
                i_x += 1;
            }
        }

        if (direction & LOWER) != NONE_DIRECTION {
            let mut i_y = y - 1;

            while board.squares[i_y][x] != put_square {
                board.squares[i_y][x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: x as u8, y: i_y as u8 });

                i_y -= 1;
            }
        }

        if (direction & LOWER_LEFT) != NONE_DIRECTION {
            let mut i_y = y - 1;
            let mut i_x = x - 1;

            while board.squares[i_y][i_x] != put_square {
                board.squares[i_y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: i_y as u8 });

                i_y -= 1;
                i_x -= 1;
            }
        }

        if (direction & LEFT) != NONE_DIRECTION {
            let mut i_x = x - 1;

            while board.squares[y][i_x] != put_square {
                board.squares[y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: y as u8 });

                i_x -= 1;
            }
        }

        if (direction & UPPER_LEFT) != NONE_DIRECTION {
            let mut i_y = y + 1;
            let mut i_x = x - 1;

            while board.squares[i_y][i_x] != put_square {
                board.squares[i_y][i_x] = put_square;

                update_log.flip[turns.count as usize].push(
                    Position { x: i_x as u8, y: i_y as u8 });

                i_y += 1;
                i_x -= 1;
            }
        }

        turn_color_state.0 = match turn_color_state.0 {
            TurnColorState::Black => TurnColorState::White,
            TurnColorState::White => TurnColorState::Black,
        };

        turns.count += 1;
        println!("turn {:?}", turn_color_state.0);
    }
}

pub fn undo (
    keyboard_input: Res<Input<KeyCode>>,
    mut board: ResMut<Board>,
    mut update_log: ResMut<UpdateLog>,
    mut turns: ResMut<Turns>,
    mut turn_color_state: ResMut<State<TurnColorState>>,
){
    if ! keyboard_input.just_pressed(KeyCode::B){
        return;
    }

    if turns.count == 0 {
        return;
    }

    turns.count -= 1;

    let put_positon = update_log.put[turns.count as usize];

    board.squares[put_positon.y as usize][put_positon.x as usize] = Square::Empty;

    for flip_position in update_log.flip[turns.count as usize].iter(){
        board.squares[flip_position.y as usize][flip_position.x as usize] =
            board.squares[flip_position.y as usize][flip_position.x as usize].invert_colors();
    }

    update_log.put.pop();
    update_log.flip.pop();


    turn_color_state.0 = match turn_color_state.0 {
        TurnColorState::Black => TurnColorState::White,
        TurnColorState::White => TurnColorState::Black,
    };

}