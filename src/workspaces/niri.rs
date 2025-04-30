use std::process;

use cosmic::iced::futures::channel::mpsc::Sender;
use cosmic::iced::futures::{self, SinkExt};
use cosmic::iced::{futures::Stream, stream};
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Event, Request, WorkspaceReferenceArg};
use std::sync::mpsc;

use super::app::Message;

pub fn sub() -> impl Stream<Item = Message> {
    let (sender, receiver) = mpsc::channel();
    let (output_sender, output_receiver) = mpsc::channel();
    tokio::task::spawn_blocking(move || apply_change(receiver));
    stream::channel(128, |mut output| async move {
        output.send(Message::Ready(sender)).await.unwrap();
        tokio::task::spawn_blocking(move || listen(output_receiver));
        output_sender.send(output).unwrap();
    })
}

fn listen(rx: mpsc::Receiver<Sender<Message>>) {
    let socket = match Socket::connect() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            // not wasting any ram
            process::exit(1);
        }
    };
    let mut event_stream = socket.send(Request::EventStream).unwrap().1;

    let mut output = rx.recv().unwrap();

    while let Ok(event) = event_stream() {
        match event {
            Event::WorkspacesChanged { workspaces } => {
                futures::executor::block_on(async {
                    output
                        .send(Message::WorkspaceUpdate(workspaces))
                        .await
                        .unwrap()
                });
            }
            Event::WorkspaceActivated { id, focused } => {
                futures::executor::block_on(async {
                    output
                        .send(Message::WorkspaceActivated { id, focused })
                        .await
                        .unwrap()
                });
            }
            _ => (),
        }
    }
}

fn apply_change(receiver: mpsc::Receiver<u64>) {
    while let Ok(id) = receiver.recv() {
        let socket = Socket::connect().unwrap();
        let _ = socket
            .send(Request::Action(Action::FocusWorkspace {
                reference: WorkspaceReferenceArg::Id(id),
            }))
            .unwrap();
    }
}
