use async_std::{
    io::Result,
    prelude::*,
    sync::{Receiver, Sender},
    task,
};
use futures::{select, FutureExt};

pub use chamomile::Config as P2pConfig;
pub use chamomile::PeerId;
use chamomile::{new_channel as p2p_new_channel, start as p2p_start, Message as P2pMessage};

use crate::group::GroupId;
use crate::{new_channel, Message};

pub(crate) async fn start(
    gid: GroupId,
    config: P2pConfig,
    send: Sender<Message>,
) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    let (p2p_send, p2p_recv) = p2p_new_channel();

    println!("config join data: {:?}", config.join_data);
    // start chamomile
    let p2p_send = p2p_start(p2p_send, config).await?;

    task::spawn(run_listen(gid, send, p2p_send, p2p_recv, out_recv));

    Ok(out_send)
}

async fn run_listen(
    gid: GroupId,
    send: Sender<Message>,
    p2p_send: Sender<P2pMessage>,
    mut p2p_recv: Receiver<P2pMessage>,
    mut out_recv: Receiver<Message>,
) -> Result<()> {
    loop {
        select! {
            msg = p2p_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from p2p: {:?}", msg);
                    match msg {
                        P2pMessage::PeerJoin(peer_addr, addr, bytes) => {
                            send.send(Message::PeerJoin(peer_addr, addr, bytes)).await;
                            // TODO Debug
                            p2p_send.send(P2pMessage::PeerJoinResult(peer_addr, true, vec![])).await;
                            // p2p_send.send(P2pMessage::PeerJoinResult(peer_addr, false, vec![])).await;
                        }
                        _ => {}
                    }
                    //send.send(msg).await;
                },
                None => break,
            },
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from outside: {:?}", msg);
                    //p2p_send.send(msg).await;
                },
                None => break,
            },
        }
    }

    drop(send);
    drop(p2p_send);
    drop(p2p_recv);
    drop(out_recv);

    Ok(())
}
