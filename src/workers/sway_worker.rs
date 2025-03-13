use swayipc::{Connection, Event, EventType, WindowEvent};

use relm4::{ComponentSender, Worker};

pub struct AsyncHandler;

#[derive(Debug)]
pub enum Input {
    Start,
}

#[derive(Debug)]
pub enum Output {
    UpdateWorkspaces(i32),
    UpdateWindows(Box<WindowEvent>),
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
                let connection = Connection::new().unwrap();
                let mut events = connection
                    .subscribe([EventType::Workspace, EventType::Window])
                    .unwrap();
                while let Some(x) = events.next() {
                    match x.unwrap() {
                        Event::Window(x) => {
                            sender.output(Output::UpdateWindows(x)).unwrap();
                        }
                        Event::Workspace(x) => {
                            sender
                                .output(Output::UpdateWorkspaces(match x.current {
                                    Some(node) => node.num.unwrap(),
                                    None => 1,
                                }))
                                .unwrap();
                        }
                        _ => {
                            continue;
                        }
                    };
                }
            }
        }
    }
}
