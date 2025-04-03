use crate::WayfireEvent;
use crate::WayfireEventType;

use relm4::{ComponentSender, Worker};
use wayfire_rs::ipc::WayfireSocket;

pub struct AsyncHandler;

#[derive(Debug)]
pub enum Input {
    Start,
}

#[derive(Debug)]
pub enum Output {
    UpdateWorkspaces(i64),
    UpdateWindows(Box<WayfireEvent>),
}

impl Worker for AsyncHandler {
    type Init = ();
    type Input = Input;
    type Output = Output;

    fn init(_init: Self::Init, sender: ComponentSender<Self>) -> Self {
        sender.input(Input::Start);

        Self
    }

    fn update(&mut self, msg: Input, sender: ComponentSender<Self>) {
        match msg {
            Input::Start => {
                let new_sender = sender.clone();
                new_sender.oneshot_command(async move {
                    let mut socket = match WayfireSocket::connect().await {
                        Ok(sock) => sock,
                        Err(e) => {
                            eprintln!("Failed to connect to Wayfire IPC socket: {}", e);
                            return;
                        }
                    };

                    let _ = socket
                        .watch(Some(vec![
                            String::from("view-focused"),
                            String::from("view-unmapped"),
                            String::from("view-mapped"),
                            String::from("wset-workspace-changed"),
                        ]))
                        .await;

                    loop {
                        let event = match socket.read_message().await {
                            Ok(x) => x,
                            Err(e) => {
                                log::error!("Failed to read event: {e}");
                                continue;
                            }
                        };
                        let event_type = match event
                            .get("event")
                            .expect("failed to read event type")
                            .as_str()
                            .expect("failed to convert event to string")
                        {
                            "view-focused" => WayfireEventType::Focus,
                            "view-unmapped" => WayfireEventType::Close,
                            "view-mapped" => WayfireEventType::New,
                            "wset-workspace-changed" => {
                                let i = event
                                    .get("wset-data")
                                    .unwrap()
                                    .get("workspace")
                                    .unwrap()
                                    .get("x")
                                    .unwrap()
                                    .as_i64()
                                    .unwrap();
                                sender.output(Output::UpdateWorkspaces(i + 1)).unwrap();
                                continue;
                            }
                            _ => unreachable!(),
                        };
                        let id = if let Some(id) =
                            event.get("view").expect("failed to get view").get("id")
                        {
                            id
                        } else {
                            log::warn!("Failed to get an id.");
                            continue;
                        }
                        .as_i64()
                        .unwrap();
                        let name = event
                            .get("view")
                            .expect("failed to get view")
                            .get("app-id")
                            .expect("failed to get name")
                            .as_str()
                            .unwrap()
                            .to_owned();

                        let _ = sender.output(Output::UpdateWindows(Box::new(WayfireEvent {
                            event_type,
                            id,
                            name,
                        })));
                    }
                });
            }
        }
    }
}
