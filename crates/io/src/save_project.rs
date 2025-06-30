//! Contains the events for saving projects and their handling systems.
use crate::document::Document;
use anyhow::Context;
use bevy::prelude::{
    BevyError, Children, Entity, Event, EventReader, Name, Query, Transform, With,
};
use bevy::prelude::{Commands, default};
use data::{Element, Layer, Level, Project};
use serialization::serialize_to;
use std::{fs::File, path::PathBuf};
use utils::{AsyncComponent, report_progress};

/// When this event is sent, the associated `project` will be fetched and saved.
/// As a reaction to this event, a system will build a [`bevy::prelude::Query`] that attempts to
/// fetch all [`data::Project`]'s [`data::Level`]s (and their descendant [`data::Layer`]s along
/// all their descendants) and then (attempts) to persist them to disk.
///
/// The [`data::Project`] fetched is queried by the [`SaveProjectEvent::project`] [`Entity`],
/// the user is responsible for only emitting "valid" entities, as this crate will assume they are,
/// and according to Bevy's own documentation, this can lead to undefined behaviour if not respected.
#[derive(Event, Debug)]
pub struct SaveProjectEvent {
    /// The [`Entity`] of the [`data::Project`] to save.
    pub(crate) project: Entity,
    /// The output path of the savefile that will be created.
    pub(crate) output: PathBuf,
}

/// This event indicates that the work of a [`SaveProjectEvent`] has completed.
#[derive(Event, Debug)]
pub struct SaveProjectCompleteEvent {
    /// The [`Entity`] of the [`data::Project`] that was saved.
    pub project: Entity,
    /// The output path of the savefile that was created.
    #[allow(
        dead_code,
        reason = "Temporarily until editor and status reporting is implemented"
    )]
    pub output: PathBuf, // TODO: remove dead_code
}

impl SaveProjectEvent {
    /// Generate a new [`SaveProjectEvent`] that can be dispatched.
    #[must_use = "This event does nothing unless you dispatch it"]
    pub fn new(project: Entity, output: PathBuf) -> Self {
        Self { project, output }
    }
}

/// Bevy system that handles [`SaveProjectEvent`] events.
#[utils::bevy_system]
pub fn handle_save_project(
    mut commands: Commands,
    mut events: EventReader<SaveProjectEvent>,
    project_query: Query<(&Name, &Children), With<Project>>,
    level_query: Query<(&Level, &Name, &Children)>,
    layer_query: Query<(&Layer, &Name, &Transform, &Children)>,
    object_query: Query<(&Element, &Name, &Transform)>,
) -> Result<(), BevyError> {
    let Some(event) = events.read().next() else {
        return Ok(());
    };

    let project = project_query.get(event.project)?;

    let entity = event.project;
    let output = event.output.clone();
    let document = Document::new(project, level_query, layer_query, object_query);
    commands.spawn(AsyncComponent::new_io(
        async move |sender| {
            let file = File::create(output.clone()).with_context(|| {
                format!("Failed to open {} for writing savefile", output.display())
            })?;
            serialize_to(&document, &default(), file)?;

            // Report completion
            report_progress(
                &sender,
                SaveProjectCompleteEvent {
                    project: entity,
                    output,
                },
            )?;
            Ok(())
        },
        |_, _| {
            // TODO: handle errors.
        },
    ));

    Ok(())
}
