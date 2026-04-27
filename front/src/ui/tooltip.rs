use crate::arret::Arret;
use bevy::picking::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Tooltip;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Tooltip,
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn on_over(
    event: On<Pointer<Over>>,
    arrets: Query<&Arret>,
    mut tooltip_q: Query<&mut Text, With<Tooltip>>,
) {
    if let Ok(arret) = arrets.get(event.entity) {
        if let Ok(mut text) = tooltip_q.single_mut() {
            **text = arret.nom.clone();
        }
    }
}

pub fn on_out(_event: On<Pointer<Out>>, mut tooltip_q: Query<&mut Text, With<Tooltip>>) {
    if let Ok(mut text) = tooltip_q.single_mut() {
        text.clear();
    }
}

pub fn move_tooltip(windows: Query<&Window>, mut node_q: Query<&mut Node, With<Tooltip>>) {
    let Ok(window) = windows.single() else { return };
    let Ok(mut node) = node_q.single_mut() else {
        return;
    };

    if let Some(pos) = window.cursor_position() {
        node.left = Val::Px(pos.x + 10.0);
        node.top = Val::Px(pos.y + 10.0);
    }
}
