use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Line},
    widgets::{
        Block, Borders, Chart, Dataset, Gauge, Paragraph,
        canvas::{self, Canvas},
    },
    Frame,
};
use crate::app::App;
use std::time::{Duration, Instant};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Create the layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
        ])
        .split(f.area());

    // Draw header
    let title = format!(" 🚨 Nuclear Reactor Monitor (CPU Load: {:.1}%) 🚨 ", app.system_info.cpu_usage);
    let title_style = Style::default()
        .fg(reactor_status_color(&app.reactor_status))
        .add_modifier(Modifier::BOLD);
    
    let header = Paragraph::new(title)
        .style(title_style)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(header, main_layout[0]);

    // Split the content area
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Left panel (reactor)
            Constraint::Percentage(40),  // Right panel (stats)
        ])
        .split(main_layout[1]);

    // Split the left panel into reactor display and history
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),  // Reactor display
            Constraint::Percentage(30),  // Temperature history
        ])
        .split(content_layout[0]);

    // Split the right panel for stats and controls
    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),  // System stats
            Constraint::Percentage(30),  // Help/controls
        ])
        .split(content_layout[1]);

    // Draw the reactor
    draw_reactor(f, app, left_layout[0]);
    
    // Draw temperature history
    draw_temperature_chart(f, app, left_layout[1]);
    
    // Draw system stats
    draw_system_stats(f, app, right_layout[0]);
    
    // Draw help if enabled, otherwise show status
    if app.show_help {
        draw_help(f, right_layout[1]);
    } else {
        draw_status(f, app, right_layout[1]);
    }
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

const EXPLOSION_COLORS: [Color; 5] = [
    Color::Red,
    Color::LightRed,
    Color::Yellow,
    Color::LightYellow,
    Color::White,
];

fn draw_reactor(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Core Stability: {:.1}%", 100.0 - app.reactor.stability()))
        .borders(Borders::ALL);
    
    f.render_widget(block, area);

    // Check if reactor is exploding
    if app.reactor.is_exploding {
        draw_explosion(f, app, area);
        return;
    }
    
    // Draw the reactor core and particles using canvas
    let reactor_canvas = Canvas::default()
        .block(Block::default())
        .x_bounds([0.0, 1.0])
        .y_bounds([0.0, 1.0])
        .paint(|ctx| {
            // Draw reactor outer casing (circle)
            ctx.draw(&canvas::Circle {
                x: 0.5,
                y: 0.5,
                radius: 0.4,
                color: Color::DarkGray,
            });
            
            // Draw coolant layer
            let coolant_color = if app.reactor.coolant_level < 50.0 {
                Color::Rgb(0, (app.reactor.coolant_level * 2.55) as u8, 255)
            } else {
                Color::Cyan
            };
            
            ctx.draw(&canvas::Circle {
                x: 0.5,
                y: 0.5,
                radius: 0.35,
                color: coolant_color,
            });
            
            // Draw reactor core (inner circle)
            let core_color = match app.reactor.stability() {
                s if s > 90.0 => Color::LightRed,
                s if s > 70.0 => Color::Red,
                s if s > 50.0 => Color::LightYellow,
                s if s > 30.0 => Color::Yellow,
                _ => Color::Green,
            };
            
            ctx.draw(&canvas::Circle {
                x: 0.5,
                y: 0.5,
                radius: 0.25,
                color: core_color,
            });
            
            // Draw control rods
            let rod_length = 0.2;
            let rod_width = 0.03;
            let rod_distance = 0.15;
            
            // Top rod
            ctx.draw(&canvas::Rectangle {
                x: 0.5 - rod_width / 2.0,
                y: 0.5 - rod_distance - rod_length * app.reactor.rod_position as f64,
                width: rod_width,
                height: rod_length * app.reactor.rod_position as f64,
                color: Color::DarkGray,
            });
            
            // Bottom rod
            ctx.draw(&canvas::Rectangle {
                x: 0.5 - rod_width / 2.0,
                y: 0.5 + rod_distance as f64,
                width: rod_width,
                height: rod_length * app.reactor.rod_position as f64,
                color: Color::DarkGray,
            });
            
            // Left rod
            ctx.draw(&canvas::Rectangle {
                x: 0.5 - rod_distance - rod_length * app.reactor.rod_position as f64,
                y: 0.5 - rod_width / 2.0,
                width: rod_length * app.reactor.rod_position as f64,
                height: rod_width,
                color: Color::DarkGray,
            });
            
            // Right rod
            ctx.draw(&canvas::Rectangle {
                x: 0.5 + rod_distance as f64,
                y: 0.5 - rod_width / 2.0,
                width: rod_length * app.reactor.rod_position as f64,
                height: rod_width,
                color: Color::DarkGray,
            });
            
            // Draw radiation particles with varying colors based on intensity
            for particle in &app.reactor.particles {
                // Calculate color based on lifetime, energy and CPU load
                let intensity = app.system_info.cpu_usage / 100.0;
                let energy_factor = particle.energy;
                
                let particle_color = if intensity > 0.7 {
                    // High CPU: more red/orange particles
                    let red_intensity = 255.min((200.0 + particle.lifetime as f32 * 2.0) as u8);
                    let green_value = ((energy_factor * 100.0) as u8).min(150);
                    Color::Rgb(red_intensity, green_value, 0)
                } else if intensity > 0.4 {
                    // Medium CPU: more orange/yellow particles
                    Color::Rgb(255, ((energy_factor * 150.0) as u8).min(255), 0)
                } else {
                    // Low CPU: yellow/green particles
                    Color::LightYellow
                };
                
                // Draw particles slightly larger if they're more energetic
                let display_radius = (0.005 + (particle.energy * 0.005)) as f64;
                
                ctx.draw(&canvas::Circle {
                    x: particle.x as f64,
                    y: particle.y as f64,
                    radius: display_radius,
                    color: particle_color,
                });
            }
            
            // Show particle count and collision info
            if app.reactor.particles.len() > 0 {
                let count_text = format!("Particles: {}", app.reactor.particles.len());
                ctx.print(0.02, 0.02, count_text);
                
                let collision_text = format!("Collisions: {}", app.reactor.collisions());
                ctx.print(0.02, 0.04, collision_text);
                
                let energy_text = format!("Core Energy: {:.1}%", app.system_info.cpu_usage);
                ctx.print(0.02, 0.06, energy_text);
            }
        });
    
    f.render_widget(reactor_canvas, inner_area(area));
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
    (value.max(0.0).min(100.0) as u16)
} 