mod brightness;
mod volume;

use std::process::Command;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;
use serde_json::json;
use wayfire_rs::ipc::WayfireSocket;

#[tracker::track]
pub struct ControlPanelModel {
    visible: bool,
    dock_enabled: bool,
    tiling: bool,
    notifs: bool,
    #[tracker::do_not_track]
    brightness: AsyncController<brightness::BrightnessModel>,
    #[tracker::do_not_track]
    volume: AsyncController<volume::VolumeModel>,
}

#[derive(Debug)]
pub enum Input {
    Toggle,
    ToggleDock,
    UpdateBrightness(u32),
    SetBrightness(u32),
    ReloadCSS,
    ToggleTiling,
    UpdateVolume(f64),
    SetVolume(f64),
    ToggleNotifs,
}

#[derive(Debug)]
pub enum Output {
    SetBrightness(u32),
    SetVolume(f64),
    ToggleDock,
}

#[relm4::component(async pub)]
impl AsyncComponent for ControlPanelModel {
    type Init = ();
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();

    view! {
        #[name = "window"]
        gtk::Window {
            #[track = "model.changed_visible()"]
            set_visible: model.visible,
            gtk::Box {
                set_margin_all: 8,
                add_css_class: "control_panel",
                set_spacing: 8,
                set_orientation: gtk::Orientation::Vertical,

                gtk::Grid {
                    set_row_spacing: 8,
                    set_column_spacing: 8,

                    attach[1, 1, 1, 1] = &gtk::Button {
                        add_css_class: "toggle_button",
                        connect_clicked => Input::ReloadCSS,
                        set_label: "Reload"
                    },
                    attach[2, 1, 1, 1] = &gtk::Button {
                        add_css_class: "toggle_button",
                        #[track = "model.changed_dock_enabled()"]
                        set_class_active: ("active", model.dock_enabled),
                        set_label: "Dock",
                        connect_clicked => Input::ToggleDock,
                    },
                    attach[1, 2, 1, 1] = &gtk::Button {
                        add_css_class: "toggle_button",
                        #[track = "model.changed_tiling()"]
                        set_class_active: ("active", model.tiling),
                        set_label: "Tiling",
                        connect_clicked => Input::ToggleTiling,
                    },
                    attach[2, 2, 1, 1] = &gtk::Button {
                        add_css_class: "toggle_button",
                        #[track = "model.changed_notifs()"]
                        set_class_active: ("active", model.notifs),
                        set_label: "Notifs",
                        connect_clicked => Input::ToggleNotifs,
                    },
                },
                model.brightness.widget(),
                model.volume.widget(),
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let brightness = brightness::BrightnessModel::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                brightness::Output::SetBrightness(x) => Input::SetBrightness(x),
            },
        );
        let volume = volume::VolumeModel::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                volume::Output::SetVolume(x) => Input::SetVolume(x),
            },
        );

        let model = Self {
            visible: false,
            dock_enabled: true,
            tiling: false,
            notifs: true,
            brightness,
            volume,
            tracker: 0,
        };

        let widgets = view_output!();

        widgets.window.init_layer_shell();
        widgets.window.set_layer(Layer::Top);
        for (anchor, state) in [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ] {
            widgets.window.set_anchor(anchor, state);
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match msg {
            Input::ToggleDock => {
                self.set_dock_enabled(!self.dock_enabled);
                sender.output(Output::ToggleDock).unwrap();
            }
            Input::Toggle => {
                self.set_visible(!self.visible);
            }
            Input::UpdateBrightness(x) => {
                self.brightness.emit(brightness::Input::Update(x));
            }
            Input::ReloadCSS => {
                //set_global_css(&crate::util::load_css("resources/bar.css"));
            }
            Input::SetBrightness(x) => {
                sender.output(Output::SetBrightness(x)).unwrap();
            }
            Input::ToggleTiling => {
                self.set_tiling(!self.tiling);

                let mut socket = match WayfireSocket::connect().await {
                    Ok(sock) => sock,
                    Err(e) => {
                        log::error!("Failed to connect to Wayfire IPC socket: {}", e);
                        return;
                    }
                };

                let plugins = match socket
                    .send_json(&wayfire_rs::models::MsgTemplate {
                        method: String::from("wayfire/get-config-option"),
                        data: Some(json!({ "option": "core/plugins" })),
                    })
                    .await
                {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Failed to query Wayfire options: {e}");
                        return;
                    }
                };
                let plugins = plugins.get("value").unwrap().as_str().unwrap().to_owned();

                self.set_tiling(!plugins.contains("simple-tile"));
                if self.tiling {
                    if let Err(e) = socket
                        .send_json(&wayfire_rs::models::MsgTemplate {
                            method: String::from("wayfire/set-config-options"),
                            data: Some(json!({ "core/plugins": plugins + " simple-tile"})),
                        })
                        .await
                    {
                        log::error!("Failed to enable tiling: {e}");
                    }
                } else {
                    if let Err(e) = socket
                        .send_json(&wayfire_rs::models::MsgTemplate {
                            method: String::from("wayfire/set-config-options"),
                            data: Some(
                                json!({ "core/plugins": plugins.replace("simple-tile", "")}),
                            ),
                        })
                        .await
                    {
                        log::error!("Failed to disable tiling: {e}");
                    }
                }
            }
            Input::UpdateVolume(x) => {
                self.volume.emit(volume::Input::Update(x));
            }
            Input::SetVolume(x) => {
                sender.output(Output::SetVolume(x)).unwrap();
            }
            Input::ToggleNotifs => {
                match Command::new("swaync-client")
                    .arg(if self.notifs { "-dn" } else { "-df" })
                    .output()
                {
                    Ok(_) => {
                        self.set_notifs(!self.notifs);
                    }
                    Err(e) => {
                        log::error!("Failed to toggle notifications: {e}");
                    }
                }
            }
        }
    }
}
