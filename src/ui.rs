use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Line, Text},
    widgets::{
        Block, Borders, Chart, Dataset, Gauge, Paragraph,
        canvas::{self, Canvas},
    },
    Frame,
};
use crate::app::App;
use std::time::{Duration, Instant};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Check if help should be shown
    if app.show_help {
        draw_help(f, f.area());
        return;
    }

    // Create the layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(f.area());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(main_layout[0]);

    let reactor_area = top_layout[0];
    let stats_area = top_layout[1];

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(main_layout[1]);

    let temp_area = bottom_layout[0];
    let core_usage_area = bottom_layout[1];  // New area for core usage display

    // Draw all components
    draw_reactor(f, app, reactor_area);
    draw_system_stats(f, app, stats_area);
    draw_temperature_chart(f, app, temp_area);
    draw_core_usage(f, app, core_usage_area);  // New function call
    draw_status(f, app, f.area());
}

// Add nuclear explosion ASCII art
const MUSHROOM_CLOUD: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣠⡴⢾⠅⠀⠀⠀⠉⠁⠉⠉⠰⡤⢀⠀⠀⠀
⠀⢀⠖⠚⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⡷⡀⠀
⠀⢞⢆⠀⠀⠀⠀⠀⠀⣀⠀⡀⠀⠀⠀⠀⠀⠀⠀⢧⠀
⠀⠀⠫⢄⣠⣤⣄⣤⣿⠛⠟⠻⢻⣿⣆⠀⢀⣠⣤⠞⠀
⠀⠀⠀⠀⠈⠉⠉⠛⢻⠀⠀⠀⢸⠿⠛⠛⠋⠉⠁⠀⠀
⠀⠀⠀⠀⠀⠀⢀⣀⠽⡀⡀⣰⠸⢤⣀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢰⡊⠀⠀⡀⢀⡁⣀⣀⠀⠀⣼⠆⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠉⠙⠛⢻⢹⠋⡋⣿⠛⠉⠁⠀⠀⠀⠀⠀
⠀⠀⠀⠀⡀⠤⠀⣰⣞⠸⠀⠇⢸⠖⡐⠄⠀⠀⠀⠀⠀
⠀⣀⣴⣾⣿⣿⣾⣠⣏⠀⡤⣠⡈⢧⣝⣷⣶⣄⡀⠀⠀
⠈⠀⠁⠈⠙⠹⠿⠿⠷⠻⡿⣿⠿⠿⠿⠻⠻⠯⠻⠖⡆
"#;

const REACTOR_CORE: &str = r#"
⠀⠀⠀⠀⠀⢀⣶⠛⠛⠛⠒⠒⠒⠒⠒⠒⠒⠛⠛⠛⠛⠋⠉⠉⠉⠉⠙⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠚⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠛⠛⣟⠳⣦⡀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⣾⣟⡉⠉⠙⢿⣿⡿⢿⣿⣿⣿⣯⡉⠉⠙⠻⣿⣟⣻⣿⡿⣿⣿⣯⡉⠉⠙⢿⣿⣿⣿⣿⣭⣿⣿⡉⠙⠛⠛⠿⣷⣶⣶⣶⣶⣶⡶⣖⣿⠂⡿⠛⠷⣦⣀⠀⠀⠀⠀
⠀⠀⠀⢀⣿⣿⣿⣿⣦⡀⠀⠙⢿⣿⣿⣽⣿⣿⣿⣦⡀⠀⠈⠛⢿⣿⣿⣿⣿⣿⣿⣦⡀⠀⠉⠻⣿⣭⣿⣿⣛⣿⢷⣤⡀⠀⠈⠙⢿⣿⣿⣿⣿⣿⣿⠀⡇⠀⣴⣿⣭⣿⣶⣤⡀
⠀⠀⢠⡿⢿⣿⣿⣿⣿⣿⣦⡀⠀⠙⠿⣿⣿⣿⣿⣿⣿⣦⡀⠀⠀⠙⢿⣿⣿⣿⢿⣿⣿⣦⡀⠀⠈⠛⢿⣿⣿⣿⣿⣿⣿⣦⣄⠀⠀⠈⠛⢿⣿⣿⣿⠀⣧⣾⣿⣿⣿⣿⡿⢺⠹
⠀⣰⣟⣀⣀⣙⣿⣿⣿⣿⣿⣿⣦⣀⣀⣈⣻⣿⣿⣿⣻⣿⣽⣦⣀⣀⣀⣙⣻⣿⣿⣿⣿⣽⣿⣷⣤⣀⣀⣙⣳⣽⣿⣿⣿⣿⣿⣿⣿⣆⣀⣀⣙⣿⣿⠀⣿⣿⣿⣿⡿⠋⠀⢸⢠
⣴⣋⣙⣩⣿⣯⣯⣿⣿⣿⣿⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⠥⠥⠤⠤⠶⠶⠶⠴⠤⠶⠦⠤⠤⠤⠤⠤⢤⣶⣤⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⣭⣿⢐⣿⣿⣿⠏⠀⠀⣠⣿⢸
⢹⠀⣿⣾⣿⣿⣿⣿⡿⣿⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⣿⠟⠁⠀⣠⣾⣿⣿⣾
⢸⠀⣿⣿⣿⣿⣿⠋⠀⣿⠀⡟⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣤⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⡇⠀⢀⣾⣿⣿⣿⣿⣿
⢸⠀⣿⣿⣿⡿⠁⠀⣀⣿⠀⡇⠀⠀⠀⠀⠀⠀⣠⠔⠫⠀⠀⣹⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⠟⢿⣲⢤⡀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⣇⣴⣿⣿⣿⣿⣿⣿⣿
⢸⠀⣿⣿⡟⠁⠀⣸⣿⣿⠀⡇⠀⠀⠀⠀⢀⡾⣇⠑⢀⣴⠟⠡⡚⢿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⢃⠙⣰⡟⢳⣯⣧⡄⠀⠀⠀⠀⠀⠀⢸⠀⣿⣿⣿⣿⣿⣿⠟⠉⣿
⢸⠀⣿⠏⠀⠀⣰⣿⣿⣿⠀⡇⠀⠀⠀⢠⠏⠢⣈⣶⠋⠀⠀⢺⠕⡸⣿⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⠳⣄⣽⢟⡼⢾⣄⡙⢿⣆⠀⠀⠀⠀⠀⢸⡇⣿⣿⣿⣿⠟⠁⠀⢠⣿
⢸⠀⡇⠀⢀⣼⣿⣿⣿⣿⢀⡇⠀⠀⣰⡏⠓⣶⠟⠀⠀⠀⠀⠀⢋⠥⠘⣷⡀⠀⠀⠀⠀⠀⠀⠀⢀⡼⡙⢢⣼⠃⠉⠀⠀⠉⠷⡄⢻⣧⠀⠀⠀⠀⢸⡇⣿⣿⡿⠃⠀⠀⣴⣿⣿
⢸⠀⡇⣠⣿⣿⣿⣿⣿⣿⢸⡇⠀⣰⢃⠈⣶⠏⠀⠀⠀⠀⠀⠀⠀⠊⡄⠘⣧⡀⠀⠀⠀⠀⠀⢠⡟⢄⣱⡾⠁⠀⠀⠀⠀⠀⠀⠙⠆⠹⣧⠀⠀⠀⢸⡇⡿⠋⠀⠀⣠⣾⣿⣿⣿
⢸⠀⣷⣿⣿⣿⣿⣯⠟⣿⢸⡇⢠⡷⣀⢹⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⣷⡀⠀⠀⠀⠀⠙⢎⣲⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⣧⠀⠀⢸⡇⡇⠀⢀⣼⣿⣿⣷⡿⣿
⢸⠀⣿⣿⢿⣿⡽⠃⠀⣿⢸⡇⢸⠣⣈⣿⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣼⣷⡴⠶⣤⡀⠀⠈⠻⡷⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⡇⠀⢸⡇⣇⣰⣿⣿⣿⣿⣿⡇⣿
⢸⠀⣿⣿⣾⠟⠀⠀⣼⡟⢸⡇⣸⢦⣈⡇⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⢀⣾⣿⠫⣈⣱⣤⣶⣿⣷⣄⠀⢹⣍⣻⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⠀⢸⡇⣿⣿⣧⣿⣿⣿⡿⠃⣿
⣸⢰⣟⡿⠁⠀⢀⣼⣿⡇⢸⡇⠻⣷⣽⠇⠀⠀⠀⠐⢚⢺⠓⡀⠀⠀⣸⢫⢋⠂⣼⡟⠅⠄⠆⠀⠹⣷⠈⠳⢌⣿⡀⣰⢶⢰⢠⠀⡀⠀⠀⠀⣸⠀⢸⡇⣿⣿⣿⣿⡿⠋⠀⢀⣿
⢼⢸⠏⠀⢀⣴⣿⣿⣿⡇⢸⡇⠀⠈⠻⠓⠒⠛⠛⠛⠛⠛⠛⠛⠛⠉⠉⠸⣦⠁⣿⡀⠀⠀⠀⠀⢠⣿⠀⠀⠀⠉⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠁⢸⡇⣿⣿⡿⠋⠀⢀⣴⣿⣿
⢻⢸⢀⣴⣿⣿⣿⣿⣿⡇⢸⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠳⢽⣷⣤⣀⣠⣴⡿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇⣿⡿⠁⠀⢠⣿⣿⣿⢻
⢸⢸⣼⣿⣿⣿⣿⣿⡟⣷⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣄⣀⣀⣀⣉⣭⠿⢥⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇⡟⠁⢀⣴⣿⣿⣿⢿⢸
⣿⢸⣿⣿⣿⣿⣿⠏⠀⣿⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣴⠯⠈⣻⣿⣯⣳⣌⣲⣄⣽⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠇⡇⢠⣿⣿⣿⣿⣿⣿⢸
⣿⢸⣿⡿⣿⠟⠁⠀⢀⣿⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⠧⡑⢠⡟⡀⡍⠉⠛⠛⡉⣅⡈⢷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣿⣧⣿⣿⡟⣿⠟⣿⢸
⣟⢸⣿⡷⠋⠀⠀⣠⣾⣿⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡾⠍⠑⢤⡟⠀⠁⠃⠆⠃⠑⠋⠁⠑⠄⢻⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣿⣿⣿⣿⡿⠃⠀⣿⢸
⣿⢸⠟⠀⠀⣠⣾⣿⡿⣿⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡞⠐⢄⣱⡿⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣿⣿⣿⠟⠁⠀⣠⣿⢸
⣿⢸⠀⣠⣾⣿⣿⣿⣿⣿⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⢦⡀⢰⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣦⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣿⣿⠏⠀⠀⣴⣿⣿⢸
⢸⢸⣾⣿⣿⣿⣿⣿⡿⣿⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠳⣤⡟⢀⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣻⣦⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⡿⠛⠀⢀⣼⣷⣿⡇⢸
⢸⢸⣿⣿⣿⣿⣿⠟⠁⣿⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠛⠷⠤⣧⣥⣀⣀⣀⣀⣀⣀⣀⣀⣤⣤⠾⠟⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠀⡇⠀⣠⢾⣿⣿⣿⣿⠸
⢸⢸⣿⣯⣿⡿⠉⠀⣠⣿⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⠉⠉⠙⠛⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⣇⣴⣿⣿⣿⣿⡿⣿⣸
⢸⠀⣿⡿⠋⠀⢀⣴⣿⣿⠀⣿⠶⢶⢶⡤⠤⠤⠤⠤⢤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⡤⠤⠤⠤⠤⠤⢤⠤⠤⠴⠶⠶⠶⠶⠶⠿⢶⠿⠿⠿⠿⠛⠛⠒⠛⢲
⢸⠀⡏⠀⠀⣰⣿⣿⣿⡏⠀⣿⣖⠚⠛⠛⠿⣶⣶⣿⣿⣿⣿⣿⣿⣟⠛⠛⠲⣶⣶⣶⣶⣷⣶⣶⣶⣖⠒⠒⠒⠲⣶⣾⣿⣿⣿⣿⣿⡛⠛⠛⠻⣾⣿⣿⣿⣷⣷⣖⠒⠒⠒⣶⡞
⢸⣀⡇⣠⣾⣿⣿⣿⣿⣇⠀⣿⣿⡷⣦⡀⠀⠉⠙⠿⣿⣿⣿⣿⣿⣾⣷⣤⡀⠀⠉⠻⣿⣿⣟⣛⣛⣿⣷⣄⠀⠀⠈⠻⢿⣿⣿⣿⣿⣿⣦⣀⠀⠈⠻⣿⣿⣿⣿⣿⣷⣄⣴⠏⠈
⠈⠛⠿⣿⣿⣿⣿⡿⠋⣿⠀⣿⣿⣿⣿⣟⣶⣄⡀⠀⠈⠛⢿⣿⣿⣿⣿⣿⣿⣦⣀⠀⠀⠻⣿⣿⣿⣿⣿⣿⣿⣦⡀⠀⠈⠛⣿⣿⣿⣿⣿⣿⡷⣄⠀⠈⠻⣿⣿⣿⣿⣿⠃⠀⠀
⠀⠀⠀⠈⠉⠻⢿⣅⡀⣿⠀⣿⣻⣿⣿⣿⣿⣾⣟⣶⣀⡀⠀⠉⠻⣿⡿⢿⣿⣿⣿⡷⣄⡀⠈⠻⣿⣿⣿⣻⣿⣿⣿⣦⡀⠀⠈⠻⣿⣿⣷⣿⣭⣿⣷⣄⠀⠀⠙⢿⣿⠃⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠻⢿⣄⣿⣀⣀⣀⣀⣀⣀⣀⣀⣉⣉⣉⣉⣉⣉⣉⣉⣉⣉⣛⣛⣛⣛⣛⣛⣛⣓⣛⠛⠛⠛⠛⠛⠛⠛⣛⣛⣛⣛⣛⣛⣛⣛⣋⣉⣉⣛⣉⣿⠃⠀⠀⠀⠀
"#;

const EXPLOSION_COLORS: [Color; 5] = [
    Color::Red,
    Color::LightRed,
    Color::Yellow,
    Color::LightYellow,
    Color::White,
];

// Add this helper function to draw a polygon with n sides
fn draw_polygon(ctx: &mut canvas::Context, x: f64, y: f64, radius: f64, sides: usize, color: Color) {
    if sides < 3 {
        return; // Can't draw polygons with less than 3 sides
    }
    
    let angle_step = std::f64::consts::TAU / sides as f64;
    let mut points = Vec::with_capacity(sides);
    
    for i in 0..sides {
        let angle = i as f64 * angle_step;
        let px = x + radius * angle.cos();
        let py = y + radius * angle.sin();
        points.push((px, py));
    }
    
    // Draw the polygon edges
    for i in 0..sides {
        let j = (i + 1) % sides;
        ctx.draw(&canvas::Line {
            x1: points[i].0,
            y1: points[i].1,
            x2: points[j].0,
            y2: points[j].1,
            color,
        });
    }
}

// Helper function to draw a filled triangle
fn draw_filled_triangle(ctx: &mut canvas::Context, p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), color: Color) {
    // Draw the triangle edges
    ctx.draw(&canvas::Line { x1: p1.0, y1: p1.1, x2: p2.0, y2: p2.1, color });
    ctx.draw(&canvas::Line { x1: p2.0, y1: p2.1, x2: p3.0, y2: p3.1, color });
    ctx.draw(&canvas::Line { x1: p3.0, y1: p3.1, x2: p1.0, y2: p1.1, color });
    
    // Draw some internal lines to make it look more filled
    let mid_p12 = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
    let mid_p23 = ((p2.0 + p3.0) / 2.0, (p2.1 + p3.1) / 2.0);
    let mid_p31 = ((p3.0 + p1.0) / 2.0, (p3.1 + p1.1) / 2.0);
    
    ctx.draw(&canvas::Line { x1: mid_p12.0, y1: mid_p12.1, x2: mid_p23.0, y2: mid_p23.1, color });
    ctx.draw(&canvas::Line { x1: mid_p23.0, y1: mid_p23.1, x2: mid_p31.0, y2: mid_p31.1, color });
    ctx.draw(&canvas::Line { x1: mid_p31.0, y1: mid_p31.1, x2: mid_p12.0, y2: mid_p12.1, color });
}

fn draw_reactor(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Core Stability: {:.1}%", 100.0 - app.reactor.stability))
        .borders(Borders::ALL);
    
    f.render_widget(block, area);

    // Check if reactor is exploding
    if app.reactor.is_exploding {
        draw_explosion(f, app, area);
        return;
    }

    // Get inner area to draw on
    let inner_area = inner_area(area);
    
    // Check if the area is too small to draw the reactor
    if inner_area.width < 10 || inner_area.height < 10 {
        return;
    }

    // Parse the ASCII art and find regions to fill with squares
    let mut reactor_lines: Vec<String> = REACTOR_CORE.lines().map(|line| line.to_string()).collect();
    
    // Find the fillable area in the ASCII art - areas containing spaces between visible parts
    // Identify open areas inside the reactor representation
    let fillable_regions = find_fillable_regions(&reactor_lines);
    
    // Get the CPU count
    let cpu_count = app.system_info.cpu_count.max(1);
    
    // Add colored squares for each CPU core
    add_cpu_squares(&mut reactor_lines, &fillable_regions, cpu_count, app);
    
    // Join the lines back together
    let reactor_art = reactor_lines.join("\n");
    
    // Create the paragraph with the ASCII art
    let paragraph = Paragraph::new(parse_ascii_with_colors(&reactor_art))
        .alignment(Alignment::Center);
    
    // Render the paragraph
    f.render_widget(paragraph, inner_area);
}

// Helper function to find fillable regions in the ASCII art
fn find_fillable_regions(lines: &[String]) -> Vec<(usize, usize)> {
    // We'll identify areas where we can place CPU cores by looking for
    // empty spaces surrounded by non-empty content
    let mut regions = Vec::new();
    
    // Find the middle section of the reactor where we want to place cores
    // We'll look for lines that have a pattern of characters, then spaces, then characters
    let core_start_line = lines.len() / 4;
    let core_end_line = lines.len() * 3 / 4;
    
    // For each line in the middle section
    for (y, line) in lines.iter().enumerate().take(core_end_line).skip(core_start_line) {
        // Skip if the line is too short
        if line.len() < 10 {
            continue;
        }
        
        // Look for regions where we have a blank space that's inside the reactor
        let mut inside_reactor = false;
        let mut current_region_start = 0;
        
        for (x, c) in line.chars().enumerate() {
            // When we find a non-space character, we're entering the reactor
            if !inside_reactor && c != ' ' {
                inside_reactor = true;
            }
            // When we find a space after being inside the reactor, we've found a potential region
            else if inside_reactor && c == ' ' {
                current_region_start = x;
                inside_reactor = false;
            }
            // When we find a non-space character after being outside, we've found the end of a region
            else if !inside_reactor && c != ' ' && current_region_start > 0 {
                // Only consider regions of reasonable size (not too small or too large)
                // Prevent overflow by ensuring x > current_region_start
                let region_width = if x > current_region_start {
                    x - current_region_start
                } else {
                    0
                };
                
                if region_width > 3 && region_width < 30 {
                    regions.push((current_region_start, y));
                }
                inside_reactor = true;
                current_region_start = 0;
            }
        }
    }
    
    regions
}

// Helper function to add colored squares representing CPU cores
fn add_cpu_squares(lines: &mut Vec<String>, regions: &[(usize, usize)], cpu_count: usize, app: &App) {
    // Don't attempt to add squares if there are no regions or no CPUs
    if regions.is_empty() || cpu_count == 0 {
        return;
    }
    
    // Determine how many squares we need to place
    let squares_to_add = cpu_count;
    
    // Distribute squares evenly among available regions
    // If we have more squares than regions, some regions will get multiple squares
    for i in 0..squares_to_add {
        // Pick a region based on the current core index
        let region_idx = i % regions.len();
        let (x, y) = regions[region_idx];
        
        // Get the core usage for this CPU core
        let core_usage = app.system_info.get_core_usage(i);
        
        // Choose a character based on CPU usage
        let square_char = match core_usage {
            u if u > 90.0 => '█', // Full block for high usage
            u if u > 70.0 => '▓', // Dark shade
            u if u > 40.0 => '▒', // Medium shade
            u if u > 10.0 => '░', // Light shade
            _ => '·',      // Dot for very low usage
        };
        
        // Add the square to the line
        if y < lines.len() && x < lines[y].len() {
            // Calculate offset within the region based on which core in region
            let cores_per_region = (squares_to_add + regions.len() - 1) / regions.len();
            let region_offset = (i / regions.len()) % cores_per_region;
            
            // Ensure we don't go out of bounds
            if x + region_offset < lines[y].len() {
                // Replace the character at the position
                let line = lines[y].clone();
                // Safely handle the substring operations to avoid panics
                let prefix = if x + region_offset > 0 {
                    &line[..(x + region_offset)]
                } else {
                    ""
                };
                
                let suffix_start = (x + region_offset + 1).min(line.len());
                let suffix = if suffix_start < line.len() {
                    &line[suffix_start..]
                } else {
                    ""
                };
                
                // Create the new line with the square character
                let new_line = format!("{}{}{}", prefix, square_char, suffix);
                lines[y] = new_line;
            }
        }
    }
}

// Helper function to parse ASCII art with colors
fn parse_ascii_with_colors(ascii_art: &str) -> Text {
    let mut styled_lines = Vec::new();
    
    for line in ascii_art.lines() {
        let mut styled_spans = Vec::new();
        
        // Process each character
        for c in line.chars() {
            let style = match c {
                '█' => Style::default().fg(Color::Red),          // Full block (high usage)
                '▓' => Style::default().fg(Color::LightRed),     // Dark shade
                '▒' => Style::default().fg(Color::Yellow),       // Medium shade
                '░' => Style::default().fg(Color::Green),        // Light shade
                '·' => Style::default().fg(Color::LightGreen),   // Dot (low usage)
                ' ' => Style::default(),                         // Space
                _ => Style::default().fg(Color::Rgb(100, 200, 255)), // Default color for reactor outlines
            };
            styled_spans.push(Span::styled(c.to_string(), style));
        }
        
        styled_lines.push(Line::from(styled_spans));
    }
    
    Text::from(styled_lines)
}

fn draw_explosion(f: &mut Frame, app: &App, area: Rect) {
    let inner = inner_area(area);
    
    // Choose color based on animation frame
    let color_index = (app.reactor.explosion_frame as usize / 2) % EXPLOSION_COLORS.len();
    let frame_color = EXPLOSION_COLORS[color_index];
    
    // Split explosion text into lines
    let mut explosion_lines = Vec::new();
    for line in MUSHROOM_CLOUD.split('\n') {
        if !line.is_empty() {
            explosion_lines.push(Line::from(
                Span::styled(line, Style::default().fg(frame_color))
            ));
        }
    }
    
    // Create paragraphs with different styles based on animation frame
    let explosion_frame = app.reactor.explosion_frame;
    let style = if explosion_frame % 2 == 0 {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    // Display warning text
    let mut text = explosion_lines;
    text.push(Line::from(""));
    text.push(Line::from(
        Span::styled(
            "⚠️ CRITICAL: MELTDOWN IN PROGRESS ⚠️", 
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
        )
    ));
    text.push(Line::from(
        Span::styled(
            format!("Core temperature: {:.1}°C", 
                    1000.0 + (app.reactor.explosion_frame as f32 * 200.0)),
            Style::default().fg(Color::LightRed)
        )
    ));
    text.push(Line::from(
        Span::styled(
            format!("Radiation level: EXTREME"),
            Style::default().fg(Color::LightRed)
        )
    ));
    text.push(Line::from(""));
    text.push(Line::from(
        Span::styled(
            "Evacuate immediately!", 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )
    ));
    
    let explosion = Paragraph::new(text)
        .style(style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default());
    
    f.render_widget(explosion, inner);
}

fn draw_temperature_chart(f: &mut Frame, app: &App, area: Rect) {
    // Create dataset from temperature history
    let temp_data: Vec<(f64, f64)> = app.reactor.history
        .iter()
        .enumerate()
        .map(|(i, &temp)| (i as f64, temp as f64))
        .collect();
    
    let datasets = vec![
        Dataset::default()
            .name("Core Temperature")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::LightRed))
            .data(&temp_data),
    ];

    let min_temp = 200.0;
    let max_temp = 1000.0;
    
    let chart = Chart::new(datasets)
        .block(Block::default().title("Temperature History").borders(Borders::ALL))
        .x_axis(
            ratatui::widgets::Axis::default()
                .title(Span::styled("Time", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, (HISTORY_SIZE - 1) as f64]),
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .title(Span::styled("°C", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([min_temp, max_temp]),
        );
    
    f.render_widget(chart, area);
}

fn draw_system_stats(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("System Metrics")
        .borders(Borders::ALL);
    
    f.render_widget(block, area);
    
    let inner = inner_area(area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);
    
    // CPU Usage
    let cpu_text = Paragraph::new("CPU Usage:");
    f.render_widget(cpu_text, chunks[0]);
    
    let cpu_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(gauge_color(app.system_info.cpu_usage)))
        .percent(safe_percentage(app.system_info.cpu_usage));
    f.render_widget(cpu_gauge, chunks[1]);
    
    // Memory Usage
    let mem_text = Paragraph::new("Memory Usage:");
    f.render_widget(mem_text, chunks[2]);
    
    let mem_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(gauge_color(app.system_info.memory_usage)))
        .percent(safe_percentage(app.system_info.memory_usage));
    f.render_widget(mem_gauge, chunks[3]);
    
    // Radiation Level
    let rad_text = Paragraph::new("Radiation Level:");
    f.render_widget(rad_text, chunks[4]);
    
    let rad_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(gauge_color(app.reactor.radiation_level)))
        .percent(safe_percentage(app.reactor.radiation_level));
    f.render_widget(rad_gauge, chunks[5]);
    
    // Core Temperature
    let temp_text = Paragraph::new(format!("Core Temperature: {:.1}°C", app.reactor.core_temperature));
    f.render_widget(temp_text, chunks[6]);
    
    let temp_percent = ((app.reactor.core_temperature - 220.0) / 780.0 * 100.0).max(0.0).min(100.0);
    let temp_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(gauge_color(temp_percent as f32)))
        .percent(safe_percentage(temp_percent));
    f.render_widget(temp_gauge, chunks[7]);
    
    // Coolant Level
    let cool_text = Paragraph::new("Coolant Level:");
    f.render_widget(cool_text, chunks[8]);
    
    let cool_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(gauge_color(100.0 - app.reactor.coolant_level)))
        .percent(safe_percentage(app.reactor.coolant_level));
    f.render_widget(cool_gauge, chunks[9]);
    
    // Reactor Status
    let status_style = Style::default()
        .fg(reactor_status_color(&app.reactor_status))
        .add_modifier(Modifier::BOLD);
    
    let status_text = Paragraph::new(format!("STATUS: {}", app.reactor_status))
        .style(status_style);
    f.render_widget(status_text, chunks[10]);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("Controls:"),
        Line::from("q - Quit"),
        Line::from("p - Pause/Resume monitoring"),
        Line::from("h - Toggle help screen"),
        Line::from("s - Toggle CPU stress test/simulation"),
        Line::from(""),
        Line::from("About:"),
        Line::from("This application visualizes your system load as a nuclear reactor."),
        Line::from("Higher CPU usage = more unstable reactor with higher radiation."),
        Line::from("The control rods move based on CPU usage and temperature increases."),
        Line::from(""),
        Line::from("Physics:"),
        Line::from("• Particles bounce off walls and each other"),
        Line::from("• When particles collide, they can create new particles"),
        Line::from("• Higher CPU = more particles and more energetic collisions"),
    ];
    
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL);
    
    let help = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(help, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mut status_text = vec![
        Line::from("Press 'h' to see controls"),
        Line::from(""),
        Line::from(format!("Status: {}", app.reactor_status)),
    ];
    
    // Add paused status
    if app.state == crate::app::AppState::Paused {
        status_text.push(Line::from("⚠️ MONITORING PAUSED ⚠️").style(Style::default().fg(Color::Yellow)));
    } else {
        status_text.push(Line::from("Active monitoring"));
    }
    
    status_text.push(Line::from(""));
    status_text.push(Line::from(format!("Press 's' to simulate CPU load")));
    status_text.push(Line::from(format!("Active particles: {}", app.reactor.particles.len())));
    status_text.push(Line::from(format!("Recent collisions: {}", app.reactor.collisions())));
    status_text.push(Line::from(format!("Total collisions: {}", app.reactor.total_collisions)));
    
    // Add meltdown warning if approaching critical mass
    if app.reactor.total_collisions > 50 && !app.reactor.is_exploding {
        // Check if we're close to or over 100 collisions
        let remaining = if app.reactor.total_collisions >= 100 {
            0
        } else {
            100 - app.reactor.total_collisions
        };
        let warning = format!("⚠️ WARNING: {} collisions until meltdown!", remaining);
        status_text.push(Line::from(warning).style(Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)));
    }
    
    // If exploding, add critical warning
    if app.reactor.is_exploding {
        status_text.push(Line::from(""));
        status_text.push(Line::from("⚠️ CRITICAL: MELTDOWN IN PROGRESS ⚠️")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)));
    }
    
    let block = Block::default()
        .title("Status")
        .borders(Borders::ALL);
    
    let status = Paragraph::new(status_text)
        .block(block)
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(status, area);
}

// Helper functions
fn inner_area(area: Rect) -> Rect {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
        ])
        .margin(1)
        .split(area)[0];
    inner
}

fn gauge_color(value: f32) -> Color {
    match value as u16 {
        0..=20 => Color::Blue,
        21..=40 => Color::Green,
        41..=60 => Color::Yellow,
        61..=80 => Color::LightRed,
        _ => Color::Red,
    }
}

fn reactor_status_color(status: &str) -> Color {
    if status.contains("Critical") || status.contains("Danger") {
        Color::Red
    } else if status.contains("Warning") {
        Color::LightYellow
    } else if status.contains("Caution") {
        Color::Yellow
    } else {
        Color::Green
    }
}

const HISTORY_SIZE: usize = 30;

// Add this helper function to ensure percentages stay within bounds
fn safe_percentage(value: f32) -> u16 {
    value.max(0.0).min(100.0) as u16
}

fn draw_core_usage(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Core Usage")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    
    f.render_widget(block.clone(), area);
    
    // Define the inner area for the core usage bars
    let inner_area = block.inner(area);
    
    // Calculate how many cores we need to display
    let core_count = app.system_info.cpu_count;
    
    // Skip if no cores or the area is too small
    if core_count == 0 || inner_area.height < 3 || inner_area.width < 3 {
        return;
    }
    
    // Calculate bar width and spacing - ensure we have at least 1 unit of width per core
    let available_width = if inner_area.width > 2 {
        (inner_area.width - 2) as usize // Account for borders
    } else {
        0
    };
    
    if available_width == 0 {
        return; // Not enough space to draw anything meaningful
    }
    
    let bar_width = (available_width / core_count).max(1);
    let spacing = if core_count > 1 && available_width > core_count * bar_width {
        (available_width - (core_count * bar_width)) / (core_count - 1)
    } else {
        0
    };
    
    // Calculate max bar height once to avoid repeated calculations
    let max_height = if inner_area.height > 2 {
        inner_area.height - 2 // Leave space for labels
    } else {
        0
    };
    
    if max_height == 0 {
        return; // Not enough vertical space for bars
    }
    
    // Draw each core usage bar
    for i in 0..core_count {
        let usage = app.system_info.get_core_usage(i);
        
        // Calculate bar position - use saturating operations to avoid overflow
        let offset = (i * (bar_width + spacing)) as u16;
        // Avoid overflow by checking the offset against available width
        let x = if offset < inner_area.width.saturating_sub(1) {
            inner_area.left().saturating_add(1).saturating_add(offset)
        } else {
            // If the offset would cause overflow, place at a safe position
            inner_area.left().saturating_add(1)
        };
        
        // Calculate bar height based on usage percentage
        let bar_height = ((usage / 100.0) * max_height as f32).round() as u16;
        let bar_height = bar_height.min(max_height);
        
        // Choose color based on usage
        let color = match usage {
            u if u > 90.0 => Color::Red,
            u if u > 70.0 => Color::LightRed,
            u if u > 50.0 => Color::Yellow,
            u if u > 30.0 => Color::Green,
            _ => Color::LightGreen,
        };
        
        // Draw the bar
        // Use saturating arithmetic for all calculations to prevent overflow
        let bottom = inner_area.bottom();
        let top = inner_area.top();
        
        // Calculate bar position with complete safety
        let y_position = if bottom > top {
            // We have at least 1 unit of height
            if bar_height.saturating_add(1) < bottom.saturating_sub(top) {
                // Normal case: bottom - (1 + bar_height)
                bottom.saturating_sub(1).saturating_sub(bar_height)
            } else {
                // Not enough room or would overflow, place at top
                top
            }
        } else {
            // Area is too small, default to top position
            top
        };
        
        // Make sure bar height doesn't exceed available space
        let safe_height = if bottom > y_position {
            bottom.saturating_sub(y_position).min(bar_height)
        } else {
            1 // Minimum height
        };
        
        // Ensure bar width doesn't overflow when converting from usize to u16
        let safe_bar_width = if bar_width > u16::MAX as usize {
            u16::MAX
        } else {
            bar_width as u16
        };
        
        let bar_rect = Rect::new(
            x,
            y_position,
            safe_bar_width,
            safe_height,
        );
        
        let bar = Block::default()
            .style(Style::default().bg(color));
        
        f.render_widget(bar, bar_rect);
        
        // Draw core number at the bottom
        let core_label = format!("{}", i);
        // Fix: Ensure the width calculation doesn't overflow
        let label_width = core_label.len().min(u16::MAX as usize) as u16;
        let label_x = if safe_bar_width > label_width {
            x.saturating_add((safe_bar_width.saturating_sub(label_width)) / 2)
        } else {
            x
        };
        
        let text = Text::from(Line::from(Span::styled(
            &core_label,
            Style::default().fg(Color::Gray),
        )));
        
        // Make sure we don't try to position a label beyond the boundaries
        // by checking the inner_area has enough height for the label
        if inner_area.height > 0 {
            let label_y = inner_area.bottom().saturating_sub(1);
            
            let label_rect = Rect::new(
                label_x,
                label_y,
                label_width,
                1,
            );
            
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center);
            
            f.render_widget(paragraph, label_rect);
        }
        
        // Draw percentage at the top if there's enough space
        // Extra safety - only attempt to draw percentage if we have enough room
        if bar_height > 1 && safe_bar_width >= 3 && inner_area.height >= 3 {
            let usage_label = format!("{:.0}%", usage);
            // Ensure the width doesn't overflow
            let usage_width = usage_label.len().min(u16::MAX as usize) as u16;
            
            // Only draw if we have enough space and valid dimensions
            if usage_width <= safe_bar_width && 
               bar_rect.height > 0 && 
               // Check if the bar position is within inner area bounds
               bar_rect.x >= inner_area.x && 
               bar_rect.y >= inner_area.y && 
               bar_rect.x < inner_area.x.saturating_add(inner_area.width) &&
               bar_rect.y < inner_area.y.saturating_add(inner_area.height) {
                // Use saturating add for safety
                let usage_x = x.saturating_add(
                    (safe_bar_width.saturating_sub(usage_width) / 2).max(0)
                );
                
                let usage_text = Text::from(Line::from(Span::styled(
                    usage_label,
                    Style::default().fg(Color::Black).bg(color),
                )));
                
                // For the percentage, we want to place it at the top of the bar
                // Calculate a safe y-position for the percentage
                let percentage_y = if bar_rect.height >= 2 {
                    // We have enough height in the bar to place percentage inside
                    bar_rect.y
                } else if bar_rect.y > inner_area.y {
                    // Try to place above the bar if there's room
                    bar_rect.y.saturating_sub(1)
                } else {
                    // No room above, use the bar position
                    bar_rect.y
                };
                
                // Only draw percentage if it's within the inner area
                if percentage_y >= inner_area.y && 
                   percentage_y < inner_area.y.saturating_add(inner_area.height) {
                    let usage_rect = Rect::new(
                        usage_x,
                        percentage_y,
                        usage_width.min(safe_bar_width),  // Ensure width fits
                        1,
                    );
                    
                    let usage_para = Paragraph::new(usage_text)
                        .alignment(Alignment::Center);
                    
                    f.render_widget(usage_para, usage_rect);
                }
            }
        }
    }
} 