#[cfg(feature = "metrics")]
use crate::metrics::NetworkMetrics;
use crate::{
    participant::C2pFrame,
    protocols::Protocols,
    types::{
        Cid, Frame, Pid, Sid, STREAM_ID_OFFSET1, STREAM_ID_OFFSET2, VELOREN_MAGIC_NUMBER,
        VELOREN_NETWORK_VERSION,
    },
};
use futures_core::task::Poll;
use futures_util::{
    task::{noop_waker, Context},
    FutureExt,
};
#[cfg(feature = "metrics")] use std::sync::Arc;
use tokio::{
    join,
    sync::{mpsc, oneshot},
};
use tracing::*;

pub(crate) struct Channel {
    cid: Cid,
    c2w_frame_r: Option<mpsc::UnboundedReceiver<Frame>>,
    read_stop_receiver: Option<oneshot::Receiver<()>>,
}

impl Channel {
    pub fn new(cid: u64) -> (Self, mpsc::UnboundedSender<Frame>, oneshot::Sender<()>) {
        let (c2w_frame_s, c2w_frame_r) = mpsc::unbounded_channel::<Frame>();
        let (read_stop_sender, read_stop_receiver) = oneshot::channel();
        (
            Self {
                cid,
                c2w_frame_r: Some(c2w_frame_r),
                read_stop_receiver: Some(read_stop_receiver),
            },
            c2w_frame_s,
            read_stop_sender,
        )
    }

    pub async fn run(
        mut self,
        protocol: Protocols,
        mut w2c_cid_frame_s: mpsc::UnboundedSender<C2pFrame>,
        mut leftover_cid_frame: Vec<C2pFrame>,
    ) {
        let c2w_frame_r = self.c2w_frame_r.take().unwrap();
        let read_stop_receiver = self.read_stop_receiver.take().unwrap();

        //reapply leftovers from handshake
        let cnt = leftover_cid_frame.len();
        trace!(?cnt, "Reapplying leftovers");
        for cid_frame in leftover_cid_frame.drain(..) {
            w2c_cid_frame_s.send(cid_frame).unwrap();
        }
        trace!(?cnt, "All leftovers reapplied");

        trace!("Start up channel");
        match protocol {
            Protocols::Tcp(tcp) => {
                join!(
                    tcp.read_from_wire(self.cid, &mut w2c_cid_frame_s, read_stop_receiver),
                    tcp.write_to_wire(self.cid, c2w_frame_r),
                );
            },
            Protocols::Udp(udp) => {
                join!(
                    udp.read_from_wire(self.cid, &mut w2c_cid_frame_s, read_stop_receiver),
                    udp.write_to_wire(self.cid, c2w_frame_r),
                );
            },
        }

        trace!("Shut down channel");
    }
}

#[derive(Debug)]
pub(crate) struct Handshake {
    cid: Cid,
    local_pid: Pid,
    secret: u128,
    init_handshake: bool,
    #[cfg(feature = "metrics")]
    metrics: Arc<NetworkMetrics>,
}

impl Handshake {
    #[cfg(debug_assertions)]
    const WRONG_NUMBER: &'static [u8] = "Handshake does not contain the magic number required by \
                                         veloren server.\nWe are not sure if you are a valid \
                                         veloren client.\nClosing the connection"
        .as_bytes();
    #[cfg(debug_assertions)]
    const WRONG_VERSION: &'static str = "Handshake does contain a correct magic number, but \
                                         invalid version.\nWe don't know how to communicate with \
                                         you.\nClosing the connection";

    pub fn new(
        cid: u64,
        local_pid: Pid,
        secret: u128,
        #[cfg(feature = "metrics")] metrics: Arc<NetworkMetrics>,
        init_handshake: bool,
    ) -> Self {
        Self {
            cid,
            local_pid,
            secret,
            #[cfg(feature = "metrics")]
            metrics,
            init_handshake,
        }
    }

    pub async fn setup(self, protocol: &Protocols) -> Result<(Pid, Sid, u128, Vec<C2pFrame>), ()> {
        let (c2w_frame_s, c2w_frame_r) = mpsc::unbounded_channel::<Frame>();
        let (mut w2c_cid_frame_s, mut w2c_cid_frame_r) = mpsc::unbounded_channel::<C2pFrame>();

        let (read_stop_sender, read_stop_receiver) = oneshot::channel();
        let handler_future =
            self.frame_handler(&mut w2c_cid_frame_r, c2w_frame_s, read_stop_sender);
        let res = match protocol {
            Protocols::Tcp(tcp) => {
                (join! {
                    tcp.read_from_wire(self.cid, &mut w2c_cid_frame_s, read_stop_receiver),
                    tcp.write_to_wire(self.cid, c2w_frame_r).fuse(),
                    handler_future,
                })
                .2
            },
            Protocols::Udp(udp) => {
                (join! {
                    udp.read_from_wire(self.cid, &mut w2c_cid_frame_s, read_stop_receiver),
                    udp.write_to_wire(self.cid, c2w_frame_r),
                    handler_future,
                })
                .2
            },
        };

        match res {
            Ok(res) => {
                let fake_waker = noop_waker();
                let mut ctx = Context::from_waker(&fake_waker);
                let mut leftover_frames = vec![];
                while let Poll::Ready(Some(cid_frame)) = w2c_cid_frame_r.poll_recv(&mut ctx) {
                    leftover_frames.push(cid_frame);
                }
                let cnt = leftover_frames.len();
                if cnt > 0 {
                    debug!(
                        ?cnt,
                        "Some additional frames got already transferred, piping them to the \
                         bparticipant as leftover_frames"
                    );
                }
                Ok((res.0, res.1, res.2, leftover_frames))
            },
            Err(()) => Err(()),
        }
    }

    async fn frame_handler(
        &self,
        w2c_cid_frame_r: &mut mpsc::UnboundedReceiver<C2pFrame>,
        mut c2w_frame_s: mpsc::UnboundedSender<Frame>,
        read_stop_sender: oneshot::Sender<()>,
    ) -> Result<(Pid, Sid, u128), ()> {
        const ERR_S: &str = "Got A Raw Message, these are usually Debug Messages indicating that \
                             something went wrong on network layer and connection will be closed";
        #[cfg(feature = "metrics")]
        let cid_string = self.cid.to_string();

        if self.init_handshake {
            self.send_handshake(&mut c2w_frame_s).await;
        }

        let frame = w2c_cid_frame_r.recv().await.map(|(_cid, frame)| frame);
        #[cfg(feature = "metrics")]
        {
            if let Some(Ok(ref frame)) = frame {
                self.metrics
                    .frames_in_total
                    .with_label_values(&[&cid_string, &frame.get_string()])
                    .inc();
            }
        }
        let r = match frame {
            Some(Ok(Frame::Handshake {
                magic_number,
                version,
            })) => {
                trace!(?magic_number, ?version, "Recv handshake");
                if magic_number != VELOREN_MAGIC_NUMBER {
                    error!(?magic_number, "Connection with invalid magic_number");
                    #[cfg(debug_assertions)]
                    self.send_raw_and_shutdown(&mut c2w_frame_s, Self::WRONG_NUMBER.to_vec())
                        .await;
                    Err(())
                } else if version != VELOREN_NETWORK_VERSION {
                    error!(?version, "Connection with wrong network version");
                    #[cfg(debug_assertions)]
                    self.send_raw_and_shutdown(
                        &mut c2w_frame_s,
                        format!(
                            "{} Our Version: {:?}\nYour Version: {:?}\nClosing the connection",
                            Self::WRONG_VERSION,
                            VELOREN_NETWORK_VERSION,
                            version,
                        )
                        .as_bytes()
                        .to_vec(),
                    )
                    .await;
                    Err(())
                } else {
                    debug!("Handshake completed");
                    if self.init_handshake {
                        self.send_init(&mut c2w_frame_s).await;
                    } else {
                        self.send_handshake(&mut c2w_frame_s).await;
                    }
                    Ok(())
                }
            },
            Some(Ok(frame)) => {
                #[cfg(feature = "metrics")]
                self.metrics
                    .frames_in_total
                    .with_label_values(&[&cid_string, frame.get_string()])
                    .inc();
                if let Frame::Raw(bytes) = frame {
                    match std::str::from_utf8(bytes.as_slice()) {
                        Ok(string) => error!(?string, ERR_S),
                        _ => error!(?bytes, ERR_S),
                    }
                }
                Err(())
            },
            Some(Err(())) => {
                info!("Protocol got interrupted");
                Err(())
            },
            None => Err(()),
        };
        if let Err(()) = r {
            if let Err(e) = read_stop_sender.send(()) {
                trace!(
                    ?e,
                    "couldn't stop protocol, probably it encountered a Protocol Stop and closed \
                     itself already, which is fine"
                );
            }
            return Err(());
        }

        let frame = w2c_cid_frame_r.recv().await.map(|(_cid, frame)| frame);
        let r = match frame {
            Some(Ok(Frame::Init { pid, secret })) => {
                debug!(?pid, "Participant send their ID");
                #[cfg(feature = "metrics")]
                self.metrics
                    .frames_in_total
                    .with_label_values(&[&cid_string, "ParticipantId"])
                    .inc();
                let stream_id_offset = if self.init_handshake {
                    STREAM_ID_OFFSET1
                } else {
                    self.send_init(&mut c2w_frame_s).await;
                    STREAM_ID_OFFSET2
                };
                info!(?pid, "This Handshake is now configured!");
                Ok((pid, stream_id_offset, secret))
            },
            Some(Ok(frame)) => {
                #[cfg(feature = "metrics")]
                self.metrics
                    .frames_in_total
                    .with_label_values(&[&cid_string, frame.get_string()])
                    .inc();
                if let Frame::Raw(bytes) = frame {
                    match std::str::from_utf8(bytes.as_slice()) {
                        Ok(string) => error!(?string, ERR_S),
                        _ => error!(?bytes, ERR_S),
                    }
                }
                Err(())
            },
            Some(Err(())) => {
                info!("Protocol got interrupted");
                Err(())
            },
            None => Err(()),
        };
        if r.is_err() {
            if let Err(e) = read_stop_sender.send(()) {
                trace!(
                    ?e,
                    "couldn't stop protocol, probably it encountered a Protocol Stop and closed \
                     itself already, which is fine"
                );
            }
        }
        r
    }

    async fn send_handshake(&self, c2w_frame_s: &mut mpsc::UnboundedSender<Frame>) {
        #[cfg(feature = "metrics")]
        self.metrics
            .frames_out_total
            .with_label_values(&[&self.cid.to_string(), "Handshake"])
            .inc();
        c2w_frame_s
            .send(Frame::Handshake {
                magic_number: VELOREN_MAGIC_NUMBER,
                version: VELOREN_NETWORK_VERSION,
            })
            .unwrap();
    }

    async fn send_init(&self, c2w_frame_s: &mut mpsc::UnboundedSender<Frame>) {
        #[cfg(feature = "metrics")]
        self.metrics
            .frames_out_total
            .with_label_values(&[&self.cid.to_string(), "ParticipantId"])
            .inc();
        c2w_frame_s
            .send(Frame::Init {
                pid: self.local_pid,
                secret: self.secret,
            })
            .unwrap();
    }

    #[cfg(debug_assertions)]
    async fn send_raw_and_shutdown(
        &self,
        c2w_frame_s: &mut mpsc::UnboundedSender<Frame>,
        data: Vec<u8>,
    ) {
        debug!("Sending client instructions before killing");
        #[cfg(feature = "metrics")]
        {
            let cid_string = self.cid.to_string();
            self.metrics
                .frames_out_total
                .with_label_values(&[&cid_string, "Raw"])
                .inc();
            self.metrics
                .frames_out_total
                .with_label_values(&[&cid_string, "Shutdown"])
                .inc();
        }
        c2w_frame_s.send(Frame::Raw(data)).unwrap();
        c2w_frame_s.send(Frame::Shutdown).unwrap();
    }
}
