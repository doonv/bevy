//! This example illustrates how to use log events in bevy.
//!
//! It creates a in-game console for viewing logs.

use bevy::log::{Level, LogEvent};
use bevy::prelude::*;

#[derive(Component)]
struct ConsoleText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            // Uncomment this to override the default log settings:
            // level: bevy::log::Level::TRACE,
            // filter: "wgpu=warn,bevy_ecs=info".to_string(),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, log_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // Setup our console UI and mark it with the `ConsoleText` component so we can use it later
    commands.spawn((
        TextBundle {
            text: Text::default(),
            ..default()
        },
        ConsoleText,
    ));
}

// This system reads all incoming logs and then outputs them to the `ConsoleText` entity
fn log_system(
    mut query: Query<&mut Text, With<ConsoleText>>,
    mut log_events: EventReader<LogEvent>,
) {
    let mut text = query.single_mut();
    for LogEvent {
        message,
        name,
        target,
        level,
        module_path,
        file,
        line,
    } in log_events.read()
    {
        // This part is just pushing a bunch of `TextSection`s to the UI.
        
        text.sections.push(TextSection {
            value: format!("file: `{file:?}`, line: {line:?} "),
            style: TextStyle {
                font_size: 16.0,
                color: Color::rgb(0.7, 0.9, 0.7),
                ..default()
            },
        });
        text.sections.push(TextSection {
            value: format!("module_path: `{module_path:?}` "),
            style: TextStyle {
                font_size: 16.0,
                color: Color::rgb(0.7, 0.7, 0.9),
                ..default()
            },
        });
        text.sections.push(TextSection {
            value: format!("target: `{target}` "),
            style: TextStyle {
                font_size: 16.0,
                color: Color::rgb(0.7, 0.9, 0.9),
                ..default()
            },
        });
        text.sections.push(TextSection {
            value: format!("name: `{name}` "),
            style: TextStyle {
                font_size: 16.0,
                color: Color::rgb(0.9, 0.7, 0.7),
                ..default()
            },
        });
        text.sections.push(TextSection {
            value: format!("{level} "),
            style: TextStyle {
                font_size: 16.0,
                color: match *level {
                    Level::TRACE => Color::PURPLE,
                    Level::DEBUG => Color::BLUE,
                    Level::INFO => Color::GREEN,
                    Level::WARN => Color::YELLOW,
                    Level::ERROR => Color::RED,
                },
                ..default()
            },
        });
        text.sections.push(TextSection {
            value: format!("{message}\n\n"),
            style: TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        });
    }
}
