use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct BrightnessModel {
    brightness: u32,
    displaying_percent: bool,
}

#[derive(Debug)]
pub enum Input {
    Update(u32),
    ToggleDisplay,
}

#[relm4::component(pub async)]
impl AsyncComponent for BrightnessModel {
    type Init = ();
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Button {
            connect_clicked => Input::ToggleDisplay,
            add_css_class: "info_button",
            set_valign: gtk::Align::Center,

            gtk::Box {
                gtk::Image {
                    #[track = "model.changed(BrightnessModel::displaying_percent())"]
                    set_visible: !model.displaying_percent,

                    #[track = "model.changed(BrightnessModel::brightness())"]
                    set_icon_name: Some(match model.brightness {
                        0..33 => "display-brightness-low-symbolic",
                        33..66 => "display-brightness-medium-symbolic",
                        66..100 => "display-brightness-high-symbolic",
                        _ => "battery-missing-symbolic"
                    }),
                },

                gtk::Label {
                    #[track = "model.changed(BrightnessModel::displaying_percent())"]
                    set_visible: model.displaying_percent,

                    #[track = "model.changed(BrightnessModel::brightness())"]
                    set_label: &format!("{}%", model.brightness),
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = BrightnessModel {
            brightness: 0,
            displaying_percent: false,
            tracker: 0,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match msg {
            Input::Update(x) => {
                self.set_brightness(x);
            }
            Input::ToggleDisplay => {
                self.set_displaying_percent(!self.displaying_percent);
            }
        }
    }
}
