use gtk::glib::DateTime;

use gtk::prelude::*;
use relm4::prelude::*;

pub struct TimeModel {
    time: DateTime,
    displaying_date: bool,
}

#[derive(Debug)]
pub enum Input {
    Update(DateTime),
    ToggleDisplay,
}

#[relm4::component(pub async)]
impl AsyncComponent for TimeModel {
    type Init = ();
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Button {
            connect_clicked => Input::ToggleDisplay,
            add_css_class: "info_button",
            set_valign: gtk::Align::Center,

            gtk::Label {
                #[watch]
                set_label: &format!("{}", if model.displaying_date {
                    model.time.format("%m/%d/%y").unwrap()
                } else {
                    model.time.format("%I:%M %p").unwrap()
                }),
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = TimeModel {
            time: DateTime::now_local().unwrap(),
            displaying_date: false,
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
        match msg {
            Input::Update(x) => {
                self.time = x;
            }
            Input::ToggleDisplay => {
                self.displaying_date = !self.displaying_date;
            }
        }
    }
}
