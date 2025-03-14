mod brightness;
mod volume;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;

#[tracker::track]
pub struct ControlPanelModel {
    visible: bool,
    dock_enabled: bool,
    tiling: bool,
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
}

#[derive(Debug)]
pub enum Output {
    SetBrightness(u32),
    SetVolume(f64),
    ToggleDock,
    ToggleTiling(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for ControlPanelModel {
    type Init = ();
    type Input = Input;
    type Output = Output;

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
                        set_label: "Notifs",
                    },
                },
                model.brightness.widget(),
                model.volume.widget(),
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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
            tiling: true,
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

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
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
                sender.output(Output::ToggleTiling(self.tiling)).unwrap();
            }
            Input::UpdateVolume(x) => {
                self.volume.emit(volume::Input::Update(x));
            }
            Input::SetVolume(x) => {
                sender.output(Output::SetVolume(x)).unwrap();
            }
        }
    }
}
