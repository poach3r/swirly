use gtk::prelude::*;
use relm4::prelude::*;

pub struct WorkspaceModel {
    index: i32,
}

#[derive(Debug)]
pub enum WorkspaceInput {
    Select(i32),
}

#[relm4::component(pub async)]
impl AsyncComponent for WorkspaceModel {
    type Init = ();
    type Input = WorkspaceInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_valign: gtk::Align::Center,
            add_css_class: "info_button",
            gtk::Label {
                #[watch]
                set_label: &format!("Workspace {}", model.index),
            }
        }
    }

    async fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self { index: 1 };
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
            WorkspaceInput::Select(i) => {
                self.index = i;
            }
        }
    }
}
