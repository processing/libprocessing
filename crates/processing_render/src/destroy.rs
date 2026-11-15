//! Deferred destroy: a destroy waits until every command buffer has drained.

use bevy::prelude::*;
use processing_core::error::Result;

use crate::render::command::CommandBuffer;

type Destroy = Box<dyn FnOnce(&mut World) -> Result<()> + Send + Sync>;

/// Destroys waiting for the queued draw commands to run.
#[derive(Resource, Default)]
pub struct PendingDestroys(Vec<Destroy>);

/// Whether any graphics still has draw commands to run.
fn commands_queued(world: &mut World) -> bool {
    world
        .query::<&CommandBuffer>()
        .iter(world)
        .any(|buffer| !buffer.commands.is_empty())
}

/// Runs `destroy` now, or once the queued draw commands have run.
pub fn request(
    world: &mut World,
    destroy: impl FnOnce(&mut World) -> Result<()> + Send + Sync + 'static,
) -> Result<()> {
    if commands_queued(world) {
        world
            .get_resource_or_insert_with(PendingDestroys::default)
            .0
            .push(Box::new(destroy));
        Ok(())
    } else {
        destroy(world)
    }
}

/// Runs the waiting destroys if no draw command can still reference them.
pub fn run_pending(world: &mut World) {
    if !world.contains_resource::<PendingDestroys>() || commands_queued(world) {
        return;
    }
    let pending = std::mem::take(&mut world.resource_mut::<PendingDestroys>().0);
    for destroy in pending {
        if let Err(error) = destroy(world) {
            debug!("deferred destroy failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::command::DrawCommand;

    #[derive(Component)]
    struct Thing;

    fn despawn(entity: Entity) -> impl FnOnce(&mut World) -> Result<()> + Send + Sync {
        move |world| {
            world.despawn(entity);
            Ok(())
        }
    }

    #[test]
    fn a_destroy_runs_at_once_when_nothing_is_queued() {
        let mut world = World::new();
        world.spawn(CommandBuffer::new());
        let thing = world.spawn(Thing).id();
        request(&mut world, despawn(thing)).unwrap();
        assert!(world.get_entity(thing).is_err());
    }

    #[test]
    fn a_destroy_waits_for_queued_commands() {
        let mut world = World::new();
        let mut buffer = CommandBuffer::new();
        buffer.push(DrawCommand::NoStroke);
        let graphics = world.spawn(buffer).id();
        let thing = world.spawn(Thing).id();

        request(&mut world, despawn(thing)).unwrap();
        run_pending(&mut world);
        assert!(
            world.get_entity(thing).is_ok(),
            "still referenced by queued commands"
        );

        world.get_mut::<CommandBuffer>(graphics).unwrap().clear();
        run_pending(&mut world);
        assert!(world.get_entity(thing).is_err());
    }
}
