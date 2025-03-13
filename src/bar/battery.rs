use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct BatteryModel {
    life: f32,
    displaying_percent: bool,
}

#[derive(Debug)]
pub enum Input {
    Update(f32),
    ToggleDisplay,
}

#[relm4::component(pub async)]
impl AsyncComponent for BatteryModel {
    type Init = ();
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Button {
            #[track = "model.changed_life()"]
            set_visible: model.life > 0.0,
            connect_clicked => Input::ToggleDisplay,
            add_css_class: "info_button",
            set_valign: gtk::Align::Center,

            gtk::Box {
                gtk::Image {
                    #[track = "model.changed(BatteryModel::displaying_percent())"]
                    set_visible: !model.displaying_percent,

                    #[track = "model.changed(BatteryModel::life())"]
                    set_icon_name: Some(match model.life {
                        0f32..10f32 => "battery-caution-symbolic",
                        10f32..20f32 => "battery-low-symbolic",
                        20f32..30f32 => "battery-level-30-symbolic",
                        30f32..40f32 => "battery-level-40-symbolic",
                        40f32..50f32 => "battery-level-50-symbolic",
                        50f32..60f32 => "battery-level-60-symbolic",
                        60f32..70f32 => "battery-level-70-symbolic",
                        70f32..80f32 => "battery-level-80-symbolic",
                        80f32..90f32 => "battery-level-90-symbolic",
                        90f32..100f32 => "battery-level-100-symbolic",
                        _ => "battery-missing-symbolic"
                    }),
                },

                gtk::Label {
                    #[track = "model.changed(BatteryModel::displaying_percent())"]
                    set_visible: model.displaying_percent,

                    #[track = "model.changed(BatteryModel::life())"]
                    set_label: &format!("{}%", model.life),
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = BatteryModel {
            life: 1.0,
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
                self.set_life(x);
            }
            Input::ToggleDisplay => {
                self.set_displaying_percent(!self.displaying_percent);
            }
        }
    }
}
