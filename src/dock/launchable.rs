use async_process::Command;

use gtk::prelude::*;
use relm4::prelude::*;

pub struct LaunchableModel {
    name: String,
    command: String,
}

#[derive(Debug, Clone)]
pub enum Input {
    Clicked,
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for LaunchableModel {
    type Init = (String, String);
    type Input = Input;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            add_css_class: "app",
            set_valign: gtk::Align::Center,
            connect_clicked => Input::Clicked,
            gtk::Image {
                set_icon_name: Some(&self.name),
                set_icon_size: gtk::IconSize::Large,
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            name: init.0,
            command: init.1,
        }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncFactorySender<Self>) {
        match msg {
            Input::Clicked => match Command::new(&self.command).output().await {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Failed run {}: {e}", self.command);
                }
            },
        }
    }
}
