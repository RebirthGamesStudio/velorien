use crate::{
    client::{GeneralStream, InGameStream},
    Server,
};
use common::{
    comp::{
        self,
        group::{self, Group, GroupManager, Invite, PendingInvites},
        ChatType, GroupManip,
    },
    msg::{InviteAnswer, ServerGeneral},
    sync,
    sync::WorldSyncExt,
};
use specs::world::WorldExt;
use std::time::{Duration, Instant};
use tracing::{error, warn};

/// Time before invite times out
const INVITE_TIMEOUT_DUR: Duration = Duration::from_secs(31);
/// Reduced duration shown to the client to help alleviate latency issues
const PRESENTED_INVITE_TIMEOUT_DUR: Duration = Duration::from_secs(30);

// TODO: turn chat messages into enums
pub fn handle_group(server: &mut Server, entity: specs::Entity, manip: GroupManip) {
    let max_group_size = server.settings().max_player_group_size;
    let state = server.state_mut();

    match manip {
        GroupManip::Invite(uid) => {
            let mut general_streams = state.ecs().write_storage::<GeneralStream>();
            let invitee =
                match state.ecs().entity_from_uid(uid.into()) {
                    Some(t) => t,
                    None => {
                        // Inform of failure
                        if let Some(general_stream) = general_streams.get_mut(entity) {
                            let _ =
                                general_stream.0.send(ChatType::Meta.server_msg(
                                    "Invite failed, target does not exist.".to_owned(),
                                ));
                        }
                        return;
                    },
                };

            let uids = state.ecs().read_storage::<sync::Uid>();

            // Check if entity is trying to invite themselves to a group
            if uids
                .get(entity)
                .map_or(false, |inviter_uid| *inviter_uid == uid)
            {
                warn!("Entity tried to invite themselves into a group");
                return;
            }

            // Disallow inviting entity that is already in your group
            let groups = state.ecs().read_storage::<Group>();
            let group_manager = state.ecs().read_resource::<GroupManager>();
            let already_in_same_group = groups.get(entity).map_or(false, |group| {
                group_manager
                    .group_info(*group)
                    .map_or(false, |g| g.leader == entity)
                    && groups.get(invitee) == Some(group)
            });
            if already_in_same_group {
                // Inform of failure
                if let Some(general_stream) = general_streams.get_mut(entity) {
                    let _ = general_stream.0.send(ChatType::Meta.server_msg(
                        "Invite failed, can't invite someone already in your group".to_owned(),
                    ));
                }
                return;
            }

            let mut pending_invites = state.ecs().write_storage::<PendingInvites>();

            // Check if group max size is already reached
            // Adding the current number of pending invites
            let group_size_limit_reached = state
                .ecs()
                .read_storage()
                .get(entity)
                .copied()
                .and_then(|group| {
                    // If entity is currently the leader of a full group then they can't invite
                    // anyone else
                    group_manager
                        .group_info(group)
                        .filter(|i| i.leader == entity)
                        .map(|i| i.num_members)
                })
                .unwrap_or(1) as usize
                + pending_invites.get(entity).map_or(0, |p| p.0.len())
                >= max_group_size as usize;
            if group_size_limit_reached {
                // Inform inviter that they have reached the group size limit
                if let Some(general_stream) = general_streams.get_mut(entity) {
                    let _ = general_stream.0.send(
                        ChatType::Meta.server_msg(
                            "Invite failed, pending invites plus current group size have reached \
                             the group size limit"
                                .to_owned(),
                        ),
                    );
                }
                return;
            }

            let agents = state.ecs().read_storage::<comp::Agent>();
            let mut invites = state.ecs().write_storage::<Invite>();

            if invites.contains(invitee) {
                // Inform inviter that there is already an invite
                if let Some(general_stream) = general_streams.get_mut(entity) {
                    let _ =
                        general_stream
                            .0
                            .send(ChatType::Meta.server_msg(
                                "This player already has a pending invite.".to_owned(),
                            ));
                }
                return;
            }

            let mut invite_sent = false;
            // Returns true if insertion was succesful
            let mut send_invite = || {
                match invites.insert(invitee, group::Invite(entity)) {
                    Err(err) => {
                        error!("Failed to insert Invite component: {:?}", err);
                        false
                    },
                    Ok(_) => {
                        match pending_invites.entry(entity) {
                            Ok(entry) => {
                                entry
                                    .or_insert_with(|| PendingInvites(Vec::new()))
                                    .0
                                    .push((invitee, Instant::now() + INVITE_TIMEOUT_DUR));
                                invite_sent = true;
                                true
                            },
                            Err(err) => {
                                error!(
                                    "Failed to get entry for pending invites component: {:?}",
                                    err
                                );
                                // Cleanup
                                invites.remove(invitee);
                                false
                            },
                        }
                    },
                }
            };

            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();

            // If client comp
            if let (Some(in_game_stream), Some(inviter)) =
                (in_game_streams.get_mut(invitee), uids.get(entity).copied())
            {
                if send_invite() {
                    let _ = in_game_stream.0.send(ServerGeneral::GroupInvite {
                        inviter,
                        timeout: PRESENTED_INVITE_TIMEOUT_DUR,
                    });
                }
            } else if agents.contains(invitee) {
                send_invite();
            } else if let Some(general_stream) = general_streams.get_mut(entity) {
                let _ = general_stream.0.send(
                    ChatType::Meta.server_msg("Can't invite, not a player or npc".to_owned()),
                );
            }

            // Notify inviter that the invite is pending
            if invite_sent {
                if let Some(in_game_stream) = in_game_streams.get_mut(entity) {
                    let _ = in_game_stream.0.send(ServerGeneral::InvitePending(uid));
                }
            }
        },
        GroupManip::Accept => {
            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let mut invites = state.ecs().write_storage::<Invite>();
            if let Some(inviter) = invites.remove(entity).and_then(|invite| {
                let inviter = invite.0;
                let mut pending_invites = state.ecs().write_storage::<PendingInvites>();
                let pending = &mut pending_invites.get_mut(inviter)?.0;
                // Check that inviter has a pending invite and remove it from the list
                let invite_index = pending.iter().position(|p| p.0 == entity)?;
                pending.swap_remove(invite_index);
                // If no pending invites remain remove the component
                if pending.is_empty() {
                    pending_invites.remove(inviter);
                }

                Some(inviter)
            }) {
                if let (Some(in_game_stream), Some(target)) =
                    (in_game_streams.get_mut(inviter), uids.get(entity).copied())
                {
                    let _ = in_game_stream.0.send(ServerGeneral::InviteComplete {
                        target,
                        answer: InviteAnswer::Accepted,
                    });
                }
                let mut group_manager = state.ecs().write_resource::<GroupManager>();
                group_manager.add_group_member(
                    inviter,
                    entity,
                    &state.ecs().entities(),
                    &mut state.ecs().write_storage(),
                    &state.ecs().read_storage(),
                    &uids,
                    |entity, group_change| {
                        in_game_streams
                            .get_mut(entity)
                            .and_then(|s| {
                                group_change
                                    .try_map(|e| uids.get(e).copied())
                                    .map(|g| (g, s))
                            })
                            .map(|(g, s)| s.0.send(ServerGeneral::GroupUpdate(g)));
                    },
                );
            }
        },
        GroupManip::Decline => {
            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let mut invites = state.ecs().write_storage::<Invite>();
            if let Some(inviter) = invites.remove(entity).and_then(|invite| {
                let inviter = invite.0;
                let mut pending_invites = state.ecs().write_storage::<PendingInvites>();
                let pending = &mut pending_invites.get_mut(inviter)?.0;
                // Check that inviter has a pending invite and remove it from the list
                let invite_index = pending.iter().position(|p| p.0 == entity)?;
                pending.swap_remove(invite_index);
                // If no pending invites remain remove the component
                if pending.is_empty() {
                    pending_invites.remove(inviter);
                }

                Some(inviter)
            }) {
                // Inform inviter of rejection
                if let (Some(in_game_stream), Some(target)) =
                    (in_game_streams.get_mut(inviter), uids.get(entity).copied())
                {
                    let _ = in_game_stream.0.send(ServerGeneral::InviteComplete {
                        target,
                        answer: InviteAnswer::Declined,
                    });
                }
            }
        },
        GroupManip::Leave => {
            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            group_manager.leave_group(
                entity,
                &mut state.ecs().write_storage(),
                &state.ecs().read_storage(),
                &uids,
                &state.ecs().entities(),
                &mut |entity, group_change| {
                    in_game_streams
                        .get_mut(entity)
                        .and_then(|s| {
                            group_change
                                .try_map(|e| uids.get(e).copied())
                                .map(|g| (g, s))
                        })
                        .map(|(g, s)| s.0.send(ServerGeneral::GroupUpdate(g)));
                },
            );
        },
        GroupManip::Kick(uid) => {
            let mut general_streams = state.ecs().write_storage::<GeneralStream>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let alignments = state.ecs().read_storage::<comp::Alignment>();

            let target = match state.ecs().entity_from_uid(uid.into()) {
                Some(t) => t,
                None => {
                    // Inform of failure
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream.0.send(
                            ChatType::Meta
                                .server_msg("Kick failed, target does not exist.".to_owned()),
                        );
                    }
                    return;
                },
            };

            // Can't kick pet
            if matches!(alignments.get(target), Some(comp::Alignment::Owned(owner)) if uids.get(target).map_or(true, |u| u != owner))
            {
                if let Some(general_stream) = general_streams.get_mut(entity) {
                    let _ = general_stream.0.send(
                        ChatType::Meta.server_msg("Kick failed, you can't kick pets.".to_owned()),
                    );
                }
                return;
            }
            // Can't kick yourself
            if uids.get(entity).map_or(false, |u| *u == uid) {
                if let Some(general_stream) = general_streams.get_mut(entity) {
                    let _ = general_stream.0.send(
                        ChatType::Meta
                            .server_msg("Kick failed, you can't kick yourself.".to_owned()),
                    );
                }
                return;
            }

            let mut groups = state.ecs().write_storage::<group::Group>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();
            // Make sure kicker is the group leader
            match groups
                .get(target)
                .and_then(|group| group_manager.group_info(*group))
            {
                Some(info) if info.leader == entity => {
                    // Remove target from group
                    group_manager.leave_group(
                        target,
                        &mut groups,
                        &state.ecs().read_storage(),
                        &uids,
                        &state.ecs().entities(),
                        &mut |entity, group_change| {
                            in_game_streams
                                .get_mut(entity)
                                .and_then(|s| {
                                    group_change
                                        .try_map(|e| uids.get(e).copied())
                                        .map(|g| (g, s))
                                })
                                .map(|(g, s)| s.0.send(ServerGeneral::GroupUpdate(g)));
                        },
                    );

                    // Tell them the have been kicked
                    if let Some(general_stream) = general_streams.get_mut(target) {
                        let _ = general_stream.0.send(
                            ChatType::Meta
                                .server_msg("You were removed from the group.".to_owned()),
                        );
                    }
                    // Tell kicker that they were succesful
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream
                            .0
                            .send(ChatType::Meta.server_msg("Player kicked.".to_owned()));
                    }
                },
                Some(_) => {
                    // Inform kicker that they are not the leader
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream.0.send(ChatType::Meta.server_msg(
                            "Kick failed: You are not the leader of the target's group.".to_owned(),
                        ));
                    }
                },
                None => {
                    // Inform kicker that the target is not in a group
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ =
                            general_stream.0.send(ChatType::Meta.server_msg(
                                "Kick failed: Your target is not in a group.".to_owned(),
                            ));
                    }
                },
            }
        },
        GroupManip::AssignLeader(uid) => {
            let mut general_streams = state.ecs().write_storage::<GeneralStream>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let target = match state.ecs().entity_from_uid(uid.into()) {
                Some(t) => t,
                None => {
                    // Inform of failure
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream.0.send(ChatType::Meta.server_msg(
                            "Leadership transfer failed, target does not exist".to_owned(),
                        ));
                    }
                    return;
                },
            };
            let groups = state.ecs().read_storage::<group::Group>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            let mut in_game_streams = state.ecs().write_storage::<InGameStream>();
            // Make sure assigner is the group leader
            match groups
                .get(target)
                .and_then(|group| group_manager.group_info(*group))
            {
                Some(info) if info.leader == entity => {
                    // Assign target as group leader
                    group_manager.assign_leader(
                        target,
                        &groups,
                        &state.ecs().entities(),
                        &state.ecs().read_storage(),
                        &uids,
                        |entity, group_change| {
                            in_game_streams
                                .get_mut(entity)
                                .and_then(|s| {
                                    group_change
                                        .try_map(|e| uids.get(e).copied())
                                        .map(|g| (g, s))
                                })
                                .map(|(g, s)| s.0.send(ServerGeneral::GroupUpdate(g)));
                        },
                    );
                    // Tell them they are the leader
                    if let Some(general_stream) = general_streams.get_mut(target) {
                        let _ = general_stream.0.send(
                            ChatType::Meta.server_msg("You are the group leader now.".to_owned()),
                        );
                    }
                    // Tell the old leader that the transfer was succesful
                    if let Some(general_stream) = general_streams.get_mut(target) {
                        let _ = general_stream.0.send(
                            ChatType::Meta
                                .server_msg("You are no longer the group leader.".to_owned()),
                        );
                    }
                },
                Some(_) => {
                    // Inform transferer that they are not the leader
                    let mut general_streams = state.ecs().write_storage::<GeneralStream>();
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream.0.send(
                            ChatType::Meta.server_msg(
                                "Transfer failed: You are not the leader of the target's group."
                                    .to_owned(),
                            ),
                        );
                    }
                },
                None => {
                    // Inform transferer that the target is not in a group
                    let mut general_streams = state.ecs().write_storage::<GeneralStream>();
                    if let Some(general_stream) = general_streams.get_mut(entity) {
                        let _ = general_stream.0.send(ChatType::Meta.server_msg(
                            "Transfer failed: Your target is not in a group.".to_owned(),
                        ));
                    }
                },
            }
        },
    }
}
