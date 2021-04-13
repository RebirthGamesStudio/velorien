use crate::{persistence::character_updater, presence::Presence, sys::SysScheduler};
use common::comp::{Inventory, SkillSet, Waypoint};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::PresenceKind;
use specs::{Join, ReadExpect, ReadStorage, Write};

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Presence>,
        ReadStorage<'a, SkillSet>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, Waypoint>,
        ReadExpect<'a, character_updater::CharacterUpdater>,
        Write<'a, SysScheduler<Self>>,
    );

    const NAME: &'static str = "persistence";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            presences,
            player_skill_set,
            player_inventories,
            player_waypoint,
            updater,
            mut scheduler,
        ): Self::SystemData,
    ) {
        if scheduler.should_run() {
            updater.batch_update(
                (
                    &presences,
                    &player_skill_set,
                    &player_inventories,
                    player_waypoint.maybe(),
                )
                    .join()
                    .filter_map(
                        |(presence, skill_set, inventory, waypoint)| match presence.kind {
                            PresenceKind::Character(id) => {
                                Some((id, skill_set, inventory, waypoint))
                            },
                            PresenceKind::Spectator => None,
                        },
                    ),
            );
        }
    }
}
