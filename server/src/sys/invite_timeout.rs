use super::SysTimer;
use crate::streams::{GetStream, InGameStream};
use common::{
    comp::group::{Invite, PendingInvites},
    msg::{InviteAnswer, ServerGeneral},
    span,
    sync::Uid,
};
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

/// This system removes timed out group invites
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)] // TODO: Pending review in #587
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Invite>,
        WriteStorage<'a, PendingInvites>,
        WriteStorage<'a, InGameStream>,
        ReadStorage<'a, Uid>,
        Write<'a, SysTimer<Self>>,
    );

    fn run(
        &mut self,
        (entities, mut invites, mut pending_invites, mut in_game_streams, uids, mut timer): Self::SystemData,
    ) {
        span!(_guard, "run", "invite_timeout::Sys::run");
        timer.start();

        let now = std::time::Instant::now();

        let timed_out_invites = (&entities, &invites)
            .join()
            .filter_map(|(invitee, Invite(inviter))| {
                // Retrieve timeout invite from pending invites
                let pending = &mut pending_invites.get_mut(*inviter)?.0;
                let index = pending.iter().position(|p| p.0 == invitee)?;

                // Stop if not timed out
                if pending[index].1 > now {
                    return None;
                }

                // Remove pending entry
                pending.swap_remove(index);

                // If no pending invites remain remove the component
                if pending.is_empty() {
                    pending_invites.remove(*inviter);
                }

                // Inform inviter of timeout
                if let (Some(in_game_stream), Some(target)) = (
                    in_game_streams.get_mut(*inviter),
                    uids.get(invitee).copied(),
                ) {
                    in_game_stream.send_unchecked(ServerGeneral::InviteComplete {
                        target,
                        answer: InviteAnswer::TimedOut,
                    });
                }

                Some(invitee)
            })
            .collect::<Vec<_>>();

        for entity in timed_out_invites {
            invites.remove(entity);
        }

        timer.end();
    }
}
