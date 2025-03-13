use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct BrightnessModel {
    brightness: u32,
}

#[derive(Debug)]
pub enum Input {
    Update(u32),
    Changed(u32),
}

#[derive(Debug)]
pub enum Output {
    SetBrightness(u32),
}

#[relm4::component(pub async)]
impl AsyncComponent for BrightnessModel {
    type Init = ();
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();

    view! {
        gtk::Box {
            add_css_class: "container",
            set_halign: gtk::Align::Fill,
            set_spacing: 4,
            set_orientation: gtk::Orientation::Vertical,

            gtk::Label {
                set_halign: gtk::Align::Center,
                set_text: "Brightness"
            },

            gtk::Scale {
                set_halign: gtk::Align::Fill,
                set_range: (0.0, 100.0),

                #[track = "model.changed(BrightnessModel::brightness())"]
                set_value: model.brightness as f64,

                connect_value_changed[sender] => move |x| {
                    sender.input(Input::Changed(x.value() as u32))
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
            tracker: 0,
        };

        let widgets = view_output!();

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
            Input::Update(x) => {
                self.set_brightness(x);
            }
            Input::Changed(x) => {
                sender.output(Output::SetBrightness(x)).unwrap();
            }
        }
    }
}
