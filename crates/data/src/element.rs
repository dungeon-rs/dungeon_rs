//! Defines the [`Element`] enum and it's implementations.

use bevy::prelude::Component;
use bevy::prelude::Visibility;

/// An [`Element`] is the lowest level of the project hierarchy.
///
/// It defines a specific element that can be drawn on the screen, commonly  represented as an asset
/// with additional metadata.
/// For portability reasons we don't store the full path of an asset, we simply store an ID which we
/// can resolve later through the `assets` module.
///
/// We don't have to store *all* information about the elements in the enum; things like translation,
/// scale, rotation, enabled, ... can all be inferred later from the components it's placed alongside.
///
/// # Examples
///
/// Here's how to spawn a simple `Element` for the Walls.
///
/// ```
/// # use bevy::prelude::*;
/// # use data::Project;
/// # use data::Level;
/// # use data::Layer;
/// # use data::Element;
/// #
/// # fn main() {
/// #   App::new()
/// #       .add_systems(Startup, spawn_project)
/// #       .run();
/// # }
/// #
/// # fn spawn_project(mut commands: Commands) {
///     commands.spawn((
///         Project::new("Roadside Inn"),
///         children![(
///             Level::new("Ground Floor"),
///             children![(
///                 Layer::new("Walls", Transform::IDENTITY),
///                 children![(
///                     Element::new_object("<hash-of-resolvable-asset"),
///                 )]
///             )]
///         )]
///     ));
/// # }
/// ```
#[derive(Component)]
#[component(immutable)]
#[require(Visibility::default())]
#[non_exhaustive]
pub enum Element {
    /// Represents an image, no pathing, patterns, ...
    Object(String),
}

impl Element {
    /// Generates a new [`Element`] for the given asset ID.
    ///
    /// **TODO**: validate assets? maybe generate a full bundle?
    #[must_use]
    pub fn new_object(asset_id: String) -> Self {
        Self::Object(asset_id)
    }
}
