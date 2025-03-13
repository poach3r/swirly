mod battery;
mod brightness;
mod time;
mod workspace;

use swayipc::WindowEvent;

use gtk::{glib::DateTime, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;

pub struct BarModel {
    workspace: AsyncController<workspace::WorkspaceModel>,
    brightness: AsyncController<brightness::BrightnessModel>,
    battery: AsyncController<battery::BatteryModel>,
    time: AsyncController<time::TimeModel>,
}

#[derive(Debug)]
pub enum Input {
    UpdateBrightness(u32),
    UpdateBattery(f32),
    UpdateWorkspaces(i32),
    UpdateWindows(Box<WindowEvent>),
    UpdateTime(DateTime),
    ToggleControlPanel,
}

#[derive(Debug)]
pub enum Output {
    ToggleControlPanel,
}

#[relm4::component(pub)]
impl SimpleComponent for BarModel {
    type Init = ();
    type Input = Input;
    type Output = Output;

    view! {
        #[name = "window"]
        gtk::Window {
            set_visible: true,

            gtk::CenterBox {
                add_css_class: "panel",
                set_vexpand: false,

                #[wrap(Some)]
                set_start_widget = &gtk::Box {
                    set_margin_all: 4,
                    model.workspace.widget(),
                },

                #[wrap(Some)]
                set_center_widget = &gtk::Box {
                    set_margin_all: 4,
                    model.time.widget(),
                },

                #[wrap(Some)]
                set_end_widget = &gtk::Box {
                    set_margin_all: 4,
                    set_spacing: 4,
                    model.brightness.widget(),
                    model.battery.widget(),
                    gtk::Button {
                        set_valign: gtk::Align::Center,
                        add_css_class: "info_button",
                        connect_clicked => Input::ToggleControlPanel,
                        gtk::Image {
                            set_icon_name: Some("open-menu-symbolic"),
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let workspace = workspace::WorkspaceModel::builder().launch(()).detach();
        //let workspaces = workspaces::WorkspacesModel::builder().launch(()).detach();
        let time = time::TimeModel::builder().launch(()).detach();
        let brightness = brightness::BrightnessModel::builder().launch(()).detach();
        let battery = battery::BatteryModel::builder().launch(()).detach();

        let model = BarModel {
            workspace,
            brightness,
            battery,
            time,
        };
        let widgets = view_output!();

        widgets.window.init_layer_shell();
        widgets.window.set_layer(Layer::Top);
        widgets.window.auto_exclusive_zone_enable();
        for (anchor, state) in [
            (Edge::Left, true),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ] {
            widgets.window.set_anchor(anchor, state);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Input::UpdateBrightness(x) => {
                self.brightness.sender().emit(brightness::Input::Update(x))
            }
            Input::UpdateBattery(x) => self.battery.sender().emit(battery::Input::Update(x)),
            Input::UpdateWindows(_x) => {}
            Input::UpdateWorkspaces(i) => {
                self.workspace.emit(workspace::WorkspaceInput::Select(i));
            }
            Input::UpdateTime(x) => {
                self.time.emit(time::Input::Update(x));
            }
            Input::ToggleControlPanel => {
                sender.output(Output::ToggleControlPanel).unwrap();
            }
        }
    }
}
