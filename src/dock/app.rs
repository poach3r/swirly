use gtk::prelude::*;
use relm4::prelude::*;

pub struct AppModel {
    pub id: i64,
    name: String,
    focused: bool,
}

#[derive(Debug)]
pub enum Output {
    Focus(i64),
}

#[derive(Debug, Clone)]
pub enum Input {
    Focus,
    Unfocus,
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for AppModel {
    type Init = (i64, String, bool);
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_class_active: ("active", self.focused),
            add_css_class: "app",
            set_valign: gtk::Align::Center,
            connect_clicked => Input::Focus,
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
            id: init.0,
            name: init.1,
            focused: init.2,
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            Input::Focus => {
                sender.output(Output::Focus(self.id)).unwrap();
            }
            Input::Unfocus => {
                self.focused = false;
            }
        }
    }
}
