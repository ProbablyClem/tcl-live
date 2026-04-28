pub mod tooltip;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, tooltip::spawn)
        .add_observer(tooltip::on_over)
        .add_observer(tooltip::on_out)
        .add_systems(Update, tooltip::move_tooltip);
}
