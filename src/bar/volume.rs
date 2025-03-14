use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct VolumeModel {
    volume: f64,
    displaying_percent: bool,
}

#[derive(Debug)]
pub enum Input {
    Update(f64),
    ToggleDisplay,
}

#[relm4::component(pub async)]
impl AsyncComponent for VolumeModel {
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
                    #[track = "model.changed_displaying_percent()"]
                    set_visible: !model.displaying_percent,

                    #[track = "model.changed_volume()"]
                    set_icon_name: Some(match model.volume {
                        0f64 => "audio-volume-muted-symbolic",
                        1f64..30f64 => "audio-volume-low-symbolic",
                        30f64..70f64 => "audio-volume-medium-symbolic",
                        70f64..100f64 => "audio-volume-high-symbolic",
                        _ => "audio-volume-overamplified-symbolic"
                    }),
                },

                gtk::Label {
                    #[track = "model.changed_displaying_percent()"]
                    set_visible: model.displaying_percent,

                    #[track = "model.changed_volume()"]
                    set_label: &format!("{}%", model.volume.round()),
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = VolumeModel {
            volume: 1.0,
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
                self.set_volume(x);
            }
            Input::ToggleDisplay => {
                self.set_displaying_percent(!self.displaying_percent);
            }
        }
    }
}
