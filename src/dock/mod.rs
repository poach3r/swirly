mod app;
mod indicator;
mod launchable;

use std::{fs::File, io::Read};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;
use swayipc::{Connection, Node, WindowChange, WindowEvent};

#[derive(serde::Deserialize, Clone)]
struct Overrides {
    original_names: Vec<String>,
    replacement_names: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
struct Launchables {
    icons: Vec<String>,
    commands: Vec<String>,
}

#[tracker::track]
pub struct DockModel {
    enabled: bool,
    visible: bool,
    #[tracker::do_not_track]
    apps: AsyncFactoryVecDeque<app::AppModel>,
    #[tracker::do_not_track]
    launchables: AsyncFactoryVecDeque<launchable::LaunchableModel>,
    #[tracker::do_not_track]
    indicator: Controller<indicator::IndicatorModel>,
    #[tracker::do_not_track]
    theme: gtk::IconTheme,
    #[tracker::do_not_track]
    overrides: Option<Overrides>,
    apps_count: usize,
}

#[derive(Debug)]
pub enum Input {
    Init,
    Enter,
    Leave,
    Toggle,
    Update(Box<WindowEvent>),
    Focus(i64),
}

#[derive(Debug)]
pub enum Output {
    Focus(i64),
}

#[relm4::component(pub)]
impl SimpleComponent for DockModel {
    type Init = ();
    type Input = Input;
    type Output = Output;

    view! {
        #[name = "window"]
        gtk::Window {
            #[track = "model.changed_visible() || model.changed_enabled()"]
            set_visible: model.visible && model.enabled,

            gtk::Box {
                set_margin_all: 8,
                set_spacing: 8,
                add_controller = gtk::EventControllerMotion {
                    connect_leave => Input::Leave,
                },

                #[local_ref]
                launchables_box -> gtk::Box {
                    add_css_class: "dock",
                    set_spacing: 8,
                },

                #[local_ref]
                apps_box -> gtk::Box {
                    #[track = "model.changed_apps_count()"]
                    set_visible: model.apps_count > 0,
                    set_spacing: 8,
                    add_css_class: "dock",
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let apps = AsyncFactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                app::Output::Focus(x) => Input::Focus(x),
            });

        let mut launchables = AsyncFactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();
        if let Some(x) = load_launchables() {
            for (i, y) in x.icons.iter().enumerate() {
                launchables
                    .guard()
                    .push_back((y.to_owned(), x.commands[i].clone()));
            }
        }

        let indicator_builder = indicator::IndicatorModel::builder();
        relm4::main_application().add_window(&indicator_builder.root);
        let indicator =
            indicator_builder
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    indicator::Output::Enter => Input::Enter,
                });

        let model = DockModel {
            enabled: true,
            visible: false,
            apps,
            launchables,
            indicator,
            theme: gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap()),
            overrides: load_overrides(),
            apps_count: 0,
            tracker: 0,
        };

        let apps_box = model.apps.widget();
        let launchables_box = model.launchables.widget();
        let widgets = view_output!();

        widgets.window.init_layer_shell();
        widgets.window.set_layer(Layer::Top);
        for (anchor, state) in [
            (Edge::Left, false),
            (Edge::Right, false),
            (Edge::Top, false),
            (Edge::Bottom, true),
        ] {
            widgets.window.set_anchor(anchor, state);
        }

        sender.input(Input::Init);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match msg {
            Input::Focus(x) => {
                sender.output(Output::Focus(x)).unwrap();
            }
            Input::Enter => {
                self.set_visible(true);
            }
            Input::Leave => {
                self.set_visible(false);
                self.indicator.emit(indicator::Input::Leave);
            }
            Input::Toggle => {
                self.set_enabled(!self.enabled);
                self.indicator.emit(indicator::Input::Toggle);
            }
            Input::Update(x) => {
                match x.change {
                    WindowChange::Close => {
                        let mut index: usize = 999;
                        for (i, app) in self.apps.guard().iter().enumerate() {
                            if let Some(app) = app {
                                if app.id == x.container.id {
                                    index = i;
                                }
                            }
                        }
                        if index == 999 {
                            return;
                        }

                        self.apps.guard().remove(index);
                    }
                    WindowChange::New => {
                        self.apps.guard().push_back((
                            x.container.id,
                            get_name(&x.container, &self.overrides),
                            x.container.focused,
                        ));
                    }
                    WindowChange::Focus => {
                        let mut index: usize = 999;
                        for (i, app) in self.apps.guard().iter().enumerate() {
                            if let Some(app) = app {
                                if app.id == x.container.id {
                                    index = i;
                                }
                            }
                        }
                        if index == 999 {
                            return;
                        }

                        self.apps.guard().remove(index);
                        self.apps.guard().broadcast(app::Input::Unfocus);
                        self.apps.guard().insert(
                            index,
                            (
                                x.container.id,
                                get_name(&x.container, &self.overrides),
                                x.container.focused,
                            ),
                        );
                    }
                    _ => (),
                }
                self.set_apps_count(self.apps.len());
            }
            Input::Init => {
                self.apps.guard().clear();
                let mut connection = Connection::new().unwrap();
                for output in connection.get_tree().unwrap().nodes.iter() {
                    if let Some(name) = &output.name {
                        if name == "__i3" {
                            continue;
                        }
                    }

                    for workspace in output.nodes.iter() {
                        for app in workspace.nodes.iter() {
                            self.apps.guard().push_back((
                                app.id,
                                get_name(app, &self.overrides),
                                app.focused,
                            ));
                        }
                        for app in workspace.floating_nodes.iter() {
                            self.apps.guard().push_back((
                                app.id,
                                get_name(app, &self.overrides),
                                app.focused,
                            ));
                        }
                    }
                }
                self.set_apps_count(self.apps.len());
            }
        }
    }
}

fn load_overrides() -> Option<Overrides> {
    let path = if let Some(x) = option_env!("XDG_CONFIG_HOME") {
        format!("{x}/swirly/overrides.toml")
    } else if let Some(x) = option_env!("HOME") {
        format!("{x}/.config/swirly/overrides.toml")
    } else {
        log::error!("Failed to find overrides.");
        return None;
    };
    let mut file = match File::open(path) {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed to read overrides: {e}");
            return None;
        }
    };

    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to read overrides: {e}");
            return None;
        }
    }

    match toml::from_str(&buf) {
        Ok(x) => Some(x),
        Err(e) => {
            log::error!("Failed to parse overrides: {e}");
            None
        }
    }
}

fn load_launchables() -> Option<Launchables> {
    let path = if let Some(x) = option_env!("XDG_CONFIG_HOME") {
        format!("{x}/swirly/launchables.toml")
    } else if let Some(x) = option_env!("HOME") {
        format!("{x}/.config/swirly/launchables.toml")
    } else {
        log::error!("Failed to find launchables.");
        return None;
    };
    let mut file = match File::open(path) {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed to read launchables: {e}");
            return None;
        }
    };

    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to read launchables: {e}");
            return None;
        }
    }

    match toml::from_str(&buf) {
        Ok(x) => Some(x),
        Err(e) => {
            log::error!("Failed to parse launchables: {e}");
            None
        }
    }
}

fn get_name(app: &Node, overrides: &Option<Overrides>) -> String {
    let name = if let Some(id) = &app.app_id {
        id.to_string()
    } else if let Some(props) = &app.window_properties {
        props.class.clone().unwrap()
    } else if let Some(name) = &app.name {
        name.to_owned()
    } else {
        String::new() // should be unreachable but im not sure
    };

    if let Some(x) = overrides {
        for (i, over) in x.original_names.iter().enumerate() {
            if *over == name {
                match x.replacement_names.get(i) {
                    Some(x) => {
                        return x.to_owned();
                    }
                    None => {
                        log::error!("Failed to find matching override for {name}.");
                        return name;
                    }
                }
            }
        }
    }

    name
}
