#![allow(dead_code)]

mod bar;
mod control_panel;
mod dock;
mod workers;

use std::process::Command;

use env_logger::Env;

use gtk::{glib::DateTime, prelude::*};
use relm4::{prelude::*, set_global_css, WorkerController};

struct AppModel {
    brightness_mode: BrightnessMode,
    bar: Controller<bar::BarModel>,
    control_panel: AsyncController<control_panel::ControlPanelModel>,
    dock: AsyncController<dock::DockModel>,
    wayfire_worker: WorkerController<workers::wayfire_worker::AsyncHandler>,
    battery_worker: WorkerController<workers::battery_worker::AsyncHandler>,
    brightness_worker: WorkerController<workers::brightness_worker::AsyncHandler>,
    time_worker: WorkerController<workers::time_worker::AsyncHandler>,
    audio_worker: WorkerController<workers::audio_worker::AsyncHandler>,
}

enum BrightnessMode {
    BrightnessCTL,
    BacklightControl,
}

#[derive(Debug)]
pub enum Input {
    SetBrightness(u32),
    UpdateBrightness(u32),
    UpdateBattery(f32),
    UpdateWorkspaces(i64),
    UpdateWindows(Box<WayfireEvent>),
    UpdateTime(DateTime),
    ToggleControlPanel,
    ToggleDock,
    UpdateVolume(f64),
    SetVolume(f64),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = Input;
    type Output = ();

    view! {
        gtk::Window {
            set_visible: true
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let wayfire_worker = workers::wayfire_worker::AsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                workers::wayfire_worker::Output::UpdateWorkspaces(i) => Input::UpdateWorkspaces(i),
                workers::wayfire_worker::Output::UpdateWindows(x) => Input::UpdateWindows(x),
            });
        let battery_worker = workers::battery_worker::AsyncHandler::builder()
            .detach_worker(match get_battery() {
                Ok(x) => Some(x),
                Err(e) => {
                    log::error!("Failed to find battery: {e}");
                    None
                }
            })
            .forward(sender.input_sender(), |msg| match msg {
                workers::battery_worker::Output::UpdateLife(i) => Input::UpdateBattery(i),
            });
        let brightness_worker = workers::brightness_worker::AsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                workers::brightness_worker::Output::UpdateBrightness(x) => {
                    Input::UpdateBrightness(x)
                }
            });
        let time_worker = workers::time_worker::AsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                workers::time_worker::Output::UpdateTime(x) => Input::UpdateTime(x),
            });
        let audio_worker = workers::audio_worker::AsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                workers::audio_worker::Output::UpdateVolume(x) => Input::UpdateVolume(x),
            });

        let app = relm4::main_application();
        let bar_builder = bar::BarModel::builder();
        let control_panel_builder = control_panel::ControlPanelModel::builder();
        let dock_builder = dock::DockModel::builder();
        app.add_window(&bar_builder.root);
        app.add_window(&control_panel_builder.root);
        app.add_window(&dock_builder.root);

        let bar = bar_builder
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                bar::Output::ToggleControlPanel => Input::ToggleControlPanel,
            });
        let control_panel =
            control_panel_builder
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    control_panel::Output::SetBrightness(x) => Input::SetBrightness(x),
                    control_panel::Output::ToggleDock => Input::ToggleDock,
                    control_panel::Output::SetVolume(x) => Input::SetVolume(x),
                });
        let dock = dock_builder.launch(()).detach();

        let model = AppModel {
            brightness_mode: BrightnessMode::BacklightControl,
            bar,
            control_panel,
            dock,
            wayfire_worker,
            battery_worker,
            brightness_worker,
            time_worker,
            audio_worker,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Input::SetBrightness(x) => match self.brightness_mode {
                BrightnessMode::BacklightControl => {
                    match backlight_control_rs::adjust_brightness_absolute(x, true) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!(
                                "Failed to set brightness: {e}. Falling back on brightnessctl."
                            );
                            self.brightness_mode = BrightnessMode::BrightnessCTL
                        }
                    }
                }
                BrightnessMode::BrightnessCTL => {
                    match Command::new("brightnessctl")
                        .args(["set", &format!("{x}%")])
                        .output()
                    {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Failed to set brightness with brightnessctl: {e}");
                        }
                    }
                }
            },
            Input::UpdateBrightness(x) => {
                self.bar.emit(bar::Input::UpdateBrightness(x));
                self.control_panel
                    .emit(control_panel::Input::UpdateBrightness(x))
            }
            Input::UpdateBattery(x) => self.bar.emit(bar::Input::UpdateBattery(x)),
            Input::UpdateWindows(x) => {
                self.dock.emit(dock::Input::Update(x));
            }
            Input::UpdateWorkspaces(i) => {
                self.bar.emit(bar::Input::UpdateWorkspaces(i));
            }
            Input::UpdateTime(x) => {
                self.bar.emit(bar::Input::UpdateTime(x));
            }
            Input::ToggleControlPanel => self.control_panel.emit(control_panel::Input::Toggle),
            Input::ToggleDock => self.dock.emit(dock::Input::Toggle),
            Input::UpdateVolume(x) => {
                self.bar.emit(bar::Input::UpdateVolume(x));
                self.control_panel
                    .emit(control_panel::Input::UpdateVolume(x));
            }
            Input::SetVolume(x) => {
                self.audio_worker
                    .emit(workers::audio_worker::Input::SetVolume(x));
            }
        }
    }
}

#[derive(Debug)]
pub enum WayfireEventType {
    New,
    Close,
    Focus,
}

#[derive(Debug)]
pub struct WayfireEvent {
    event_type: WayfireEventType,
    id: i64,
    name: String,
}

fn main() {
    let css = include_str!("../resources/bar.css");

    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let app = RelmApp::new("org.poach3r.swirly");

    set_global_css(css);
    app.run::<AppModel>(());
}

fn get_battery() -> Result<starship_battery::Battery, starship_battery::Error> {
    Ok(starship_battery::Manager::new()?
        .batteries()?
        .last()
        .unwrap()?)
}
