use swayipc::{Connection, Fallible};

use relm4::{ComponentSender, Worker};

pub struct AsyncHandler {
    connection: Connection,
}

#[derive(Debug)]
pub enum Input {
    ToggleTiling(bool),
    Focus(i64),
    ArbitrarySwayMsg(String),
}

#[derive(Debug)]
pub enum Output {}

fn handle_swaymsg(err_msg: &str, result: Fallible<Vec<Fallible<()>>>) {
    match result {
        Ok(outcomes) => {
            for outcome in outcomes {
                if let Err(e) = outcome {
                    log::error!("{err_msg}: {e}");
                }
            }
        }
        Err(e) => {
            log::error!("{err_msg}: {e}");
        }
    }
}

impl Worker for AsyncHandler {
    type Init = ();
    type Input = Input;
    type Output = Output;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            connection: Connection::new().unwrap(),
        }
    }

    fn update(&mut self, msg: Input, _sender: ComponentSender<Self>) {
        match msg {
            Input::ArbitrarySwayMsg(x) => {
                handle_swaymsg("Failed to run command", self.connection.run_command(x));
            }
            Input::Focus(x) => {
                handle_swaymsg(
                    "Failed to focus window",
                    self.connection.run_command(format!("[con_id={x}] focus")),
                );
            }
            Input::ToggleTiling(tiling) => {
                if tiling {
                    // Using something like `for_window [app_id=".*"] floating disable`
                    // will prevent tiling from being disabled
                    // so we need to reload the config
                    // in order to disable floating.
                    handle_swaymsg(
                        "Failed to disable floating",
                        self.connection.run_command("reload"),
                    );
                } else {
                    handle_swaymsg(
                        "Failed to enable floating",
                        self.connection
                            .run_command("for_window [app_id=\".*\"] floating enable"),
                    );
                }
            }
        }
    }
}
