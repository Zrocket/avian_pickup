#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]

use avian3d::prelude::*;
use bevy::prelude::*;

mod camera;
mod config;
mod event;
mod spatial_query;
mod util;

/// Everything you need to get started with Avian Pickup.
pub mod prelude {
    pub(crate) use avian3d::prelude::*;
    pub(crate) use bevy::prelude::*;

    pub(crate) use crate::util::*;
    pub use crate::{
        camera::AvianPickupCamera,
        config::AvianPickupConfig,
        event::AvianPickupEvent,
        AvianPickupPlugin,
        AvianPickupSystem,
    };
}

/// The Avian Pickup plugin. Add this after the Avian Physics plugins to enable
/// pickup functionality. Uses the same [`Schedule`]` as Avian.
///
/// # Example
///
/// ```
/// # use avian3d::prelude::*;
/// # use avian_pickup::prelude::*;
/// # use bevy::prelude::*;
///
/// App::new().add_plugins((
///     DefaultPlugins,
///     PhysicsPlugins::default(),
///     AvianPickupPlugin::default(),
/// ));
/// ```
#[derive(Default)]
#[non_exhaustive]
pub struct AvianPickupPlugin;

impl Plugin for AvianPickupPlugin {
    fn build(&self, app: &mut App) {
        // Run `expect` first so that other plugins can just call `unwrap`.
        let physics_schedule = app.get_schedule_mut(PhysicsSchedule).expect(
            "Failed to build `AvianPickupPlugin`:\
                Avian's `PhysicsSchedule` was not found. Make sure to add Avian's plugins *before* `AvianPickupPlugin`.\
                This usually done by adding `PhysicsPlugins` to your `App`.",
        );

        physics_schedule.configure_sets(
            (AvianPickupSystem::First, AvianPickupSystem::SpatialQuery)
                .chain()
                .in_set(PhysicsStepSet::First),
        );
        app.add_plugins((
            config::plugin,
            camera::plugin,
            event::plugin,
            spatial_query::plugin,
        ));
    }
}

/// Set enum for the systems added by [`AvianPickupPlugin`].
/// Use this to order your systems relative to the ones used by Avian Pickup.
/// This is run in Avian's `PhysicsStepSet::First`.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum AvianPickupSystem {
    /// Runs at the start of the [`AvianPickupSystem`]. Empty by default.
    First,
    /// Performs spatial queries.
    SpatialQuery,
}
