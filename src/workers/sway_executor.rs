use swayipc::Connection;

use relm4::{ComponentSender, Worker};

pub struct AsyncHandler {
    connection: Connection,
}

#[derive(Debug)]
pub enum Input {
    ToggleTiling(bool),
    Focus(i64),
}

#[derive(Debug)]
pub enum Output {}

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
            Input::Focus(x) => match self.connection.run_command(format!("[con_id={x}] focus")) {
                Ok(outcomes) => {
                    for outcome in outcomes {
                        if let Err(e) = outcome {
                            log::error!("Failed to focus window: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to focus window: {e}");
                }
            },
            Input::ToggleTiling(tiling) => {
                if tiling {
                    // Using something like `for_window [app_id=".*"] floating disable`
                    // will prevent tiling from being disabled
                    // so we need to reload the config
                    // in order to disable floating.
                    match self.connection.run_command("reload") {
                        Ok(outcomes) => {
                            for outcome in outcomes {
                                if let Err(e) = outcome {
                                    log::error!("Failed to disable floating: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to disable floating: {e}");
                        }
                    }
                } else {
                    match self
                        .connection
                        .run_command("for_window [app_id=\".*\"] floating enable")
                    {
                        Ok(outcomes) => {
                            for outcome in outcomes {
                                if let Err(e) = outcome {
                                    log::error!("Failed to enable floating: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to enable floating: {e}");
                        }
                    }
                }
            }
        }
    }
}
