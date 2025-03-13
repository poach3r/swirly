use std::time::Duration;

use relm4::{ComponentSender, Worker};

pub struct AsyncHandler;

#[derive(Debug)]
pub enum Input {
    Start,
}

#[derive(Debug)]
pub enum Output {
    UpdateBrightness(u32),
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
        let interval = Duration::from_secs(5);
        match msg {
            Input::Start => loop {
                let brightness = match backlight_control_rs::get_brightness() {
                    Ok(x) => x,
                    Err(e) => {
                        log::warn!("Failed to find brightness: {e}");
                        return;
                    }
                };
                let max_brightness = match backlight_control_rs::get_max_brightness() {
                    Ok(x) => x,
                    Err(e) => {
                        log::warn!("Failed to find max brightness: {e}");
                        return;
                    }
                };

                sender
                    .output(Output::UpdateBrightness(
                        ((brightness as f32 / max_brightness as f32) * 100.0) as u32,
                    ))
                    .unwrap();

                std::thread::sleep(interval);
            },
        }
    }
}
