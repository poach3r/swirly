use starship_battery::Battery;

use std::time::Duration;

use relm4::{ComponentSender, Worker};

pub struct AsyncHandler {
    battery: Option<Battery>,
}

#[derive(Debug)]
pub enum Input {
    Start,
}

#[derive(Debug)]
pub enum Output {
    UpdateLife(f32),
}

impl Worker for AsyncHandler {
    type Init = Option<Battery>;
    type Input = Input;
    type Output = Output;

    fn init(battery: Self::Init, sender: ComponentSender<Self>) -> Self {
        sender.input(Input::Start);
        Self { battery }
    }

    fn update(&mut self, msg: Input, sender: ComponentSender<Self>) {
        let interval = Duration::from_secs(5);
        match msg {
            Input::Start => {
                if let Some(battery) = self.battery.as_mut() {
                    loop {
                        match battery.refresh() {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("Failed to refresh battery: {e}");
                                return;
                            }
                        }
                        let bat =
                            (battery.energy().value / battery.energy_full().value * 100.0).round();
                        sender.output(Output::UpdateLife(bat)).unwrap();
                        std::thread::sleep(interval);
                    }
                } else {
                    sender.output(Output::UpdateLife(-999.0)).unwrap();
                }
            }
        }
    }
}
