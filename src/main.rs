#![allow(dead_code)]

mod bar;
mod control_panel;
mod dock;
mod util;
mod workers;

use std::process::Command;

use env_logger::Env;
use swayipc::WindowEvent;

use gtk::{glib::DateTime, prelude::*};
use relm4::{prelude::*, set_global_css, WorkerController};

struct AppModel {
    brightness_mode: BrightnessMode,
    bar: Controller<bar::BarModel>,
    control_panel: Controller<control_panel::ControlPanelModel>,
    dock: Controller<dock::DockModel>,
    sway_worker: WorkerController<workers::sway_worker::AsyncHandler>,
    sway_executor: WorkerController<workers::sway_executor::AsyncHandler>,
    battery_worker: WorkerController<workers::battery_worker::AsyncHandler>,
    brightness_worker: WorkerController<workers::brightness_worker::AsyncHandler>,
    time_worker: WorkerController<workers::time_worker::AsyncHandler>,
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
    UpdateWorkspaces(i32),
    UpdateWindows(Box<WindowEvent>),
    UpdateTime(DateTime),
    ToggleControlPanel,
    ToggleDock,
    ToggleTiling(bool),
    FocusWindow(i64),
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
        let sway_worker = workers::sway_worker::AsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                workers::sway_worker::Output::UpdateWorkspaces(i) => Input::UpdateWorkspaces(i),
                workers::sway_worker::Output::UpdateWindows(x) => Input::UpdateWindows(x),
            });
        let sway_executor = workers::sway_executor::AsyncHandler::builder()
            .detach_worker(())
            .detach();
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
                    control_panel::Output::ToggleTiling(x) => Input::ToggleTiling(x),
                    control_panel::Output::SetBrightness(x) => Input::SetBrightness(x),
                    control_panel::Output::ToggleDock => Input::ToggleDock,
                });
        let dock = dock_builder
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                dock::Output::Focus(x) => Input::FocusWindow(x),
            });

        let model = AppModel {
            brightness_mode: BrightnessMode::BacklightControl,
            bar,
            control_panel,
            dock,
            sway_worker,
            sway_executor,
            battery_worker,
            brightness_worker,
            time_worker,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Input::ToggleTiling(x) => {
                self.sway_executor
                    .emit(workers::sway_executor::Input::ToggleTiling(x));
            }
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
            Input::FocusWindow(x) => self
                .sway_executor
                .emit(workers::sway_executor::Input::Focus(x)),
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let app = RelmApp::new("org.poach3r.swirly");

    set_global_css(&crate::util::load_css("resources/bar.css"));
    app.run::<AppModel>(());
}

fn get_battery() -> Result<starship_battery::Battery, starship_battery::Error> {
    Ok(starship_battery::Manager::new()?
        .batteries()?
        .last()
        .unwrap()?)
}
