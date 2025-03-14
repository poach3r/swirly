mod poller;
mod utils;

use libpulse_binding::context::{Context, State};
use libpulse_binding::error::PAErr;
use libpulse_binding::mainloop::standard::Mainloop;
use libpulse_binding::volume::{ChannelVolumes, Volume};
use relm4::{Component, ComponentSender, Worker, WorkerController};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use utils::percentage_to_volume;

pub struct AsyncHandler {
    poller: WorkerController<poller::AsyncHandler>,
    main_loop: MyMainLoop,
    context: MyContext,
}

struct MyMainLoop {
    val: Arc<Mutex<Option<Mainloop>>>,
}

struct MyContext {
    val: Arc<Mutex<Option<Context>>>,
}

unsafe impl Send for MyMainLoop {}
unsafe impl Send for MyContext {}

#[derive(Debug)]
pub enum Input {
    SetVolume(f64),
    GetVolume,
}

#[derive(Debug)]
pub enum Output {
    UpdateVolume(f64),
}

impl Worker for AsyncHandler {
    type Init = ();
    type Input = Input;
    type Output = Output;

    fn init(_init: Self::Init, sender: ComponentSender<Self>) -> Self {
        let mut main_loop = match Mainloop::new()
            .ok_or_else(|| log::error!("Failed to initialize PulseAudio main loop."))
        {
            Ok(x) => Some(x),
            Err(_) => None,
        };
        let context = if let Some(x) = main_loop.as_mut() {
            if let Ok(y) = utils::connect(x) {
                Some(y)
            } else {
                log::error!("Failed to connect to PulseAudio.");
                None
            }
        } else {
            None
        };
        let poller = poller::AsyncHandler::builder().detach_worker(()).forward(
            sender.input_sender(),
            |msg| match msg {
                poller::Output::Poll => Input::GetVolume,
            },
        );

        poller.emit(poller::Input::Start);

        Self {
            poller,
            main_loop: MyMainLoop {
                val: Arc::new(Mutex::new(main_loop)),
            },
            context: MyContext {
                val: Arc::new(Mutex::new(context)),
            },
        }
    }

    fn update(&mut self, msg: Input, sender: ComponentSender<Self>) {
        match msg {
            Input::SetVolume(x) => {
                let mut context_guard = if let Ok(x) = self.context.val.lock() {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio context.");
                    return;
                };

                let context = if let Some(x) = &mut *context_guard {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio context.");
                    return;
                };

                let mut main_loop_guard = if let Ok(x) = self.main_loop.val.lock() {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio Mainloop.");
                    return;
                };

                let main_loop = if let Some(x) = &mut *main_loop_guard {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio Mainloop.");
                    return;
                };

                if let Ok(volumes) = utils::get_output_volumes(main_loop, context).as_mut() {
                    utils::map_volumes(volumes, |_| x);
                    match utils::set_output_volumes(main_loop, context, volumes) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Failed to set output volume: {e}");
                        }
                    }
                } else {
                    log::warn!("Failed to get outputs from PulseAudio.");
                }
            }
            Input::GetVolume => {
                let mut context_guard = if let Ok(x) = self.context.val.lock() {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio context.");
                    return;
                };

                let context = if let Some(x) = &mut *context_guard {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio context.");
                    return;
                };

                let mut main_loop_guard = if let Ok(x) = self.main_loop.val.lock() {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio Mainloop.");
                    return;
                };

                let main_loop = if let Some(x) = &mut *main_loop_guard {
                    x
                } else {
                    log::warn!("Failed to get PulseAudio Mainloop.");
                    return;
                };

                if let Ok(volumes) = utils::get_output_volumes(main_loop, context) {
                    sender
                        .output(Output::UpdateVolume(utils::volume_to_percentage(
                            volumes.get()[0],
                        )))
                        .unwrap();
                } else {
                    log::warn!("Failed to get outputs from PulseAudio.");
                }
            }
        }
    }
}
