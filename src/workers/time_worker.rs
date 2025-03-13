use std::time::Duration;

use relm4::{gtk::glib::DateTime, ComponentSender, Worker};

pub struct AsyncHandler;

#[derive(Debug)]
pub enum Input {
    Start,
}

#[derive(Debug)]
pub enum Output {
    UpdateTime(DateTime),
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
        let interval = Duration::from_secs(2);
        match msg {
            Input::Start => loop {
                let time = match DateTime::now_local() {
                    Ok(t) => t,
                    Err(e) => panic!("Failed to find local time: {e}"),
                };
                sender.output(Output::UpdateTime(time)).unwrap();
                std::thread::sleep(interval);
            },
        }
    }
}
