// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Action, Application, Element,
    app::{Core, Task},
    applet::padded_control,
    cosmic_config::{ConfigGet, ConfigSet},
    iced::{Limits, window::Id},
    iced_widget::Scrollable,
    iced_winit::commands::popup::{destroy_popup, get_popup},
    widget::{self},
};
use cosmic_settings_config::window_rules;

use crate::{fl, wayland::toplevel::ToplevelsInfo};

#[derive(Default)]
pub struct App {
    core: Core,
    popup: Option<Id>,
    active_view: Views,
    apps_info: Vec<ToplevelsInfo>,
}

#[derive(Debug, Default)]
pub enum Views {
    #[default]
    Main,
    Manage(usize),
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    RefreshToplevels,
    ChangeView(usize),
    BackToMain,
    AddWithTitle(String),
    AddWithAPPID(String),
}

impl Application for App {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.heppen.tiling_exception_custom";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = App {
            core,
            ..Default::default()
        };

        (
            app,
            Task::future(async move { Action::App(Message::RefreshToplevels) }),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("appointment-new-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let view = match self.active_view {
            Views::Main => self.list_view(),
            Views::Manage(idx) => self.manage_view(idx),
        };

        self.core
            .applet
            .popup_container(view)
            .limits(Limits::NONE.max_width(720.0).max_height(360.0))
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    self.active_view = Views::Main;
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::RefreshToplevels => {
                self.apps_info = crate::wayland::toplevel::refresh_toplevels();
            }
            Message::BackToMain => self.active_view = Views::Main,
            Message::ChangeView(toplevel_index) => match &self.active_view {
                Views::Main => self.active_view = Views::Manage(toplevel_index),
                Views::Manage(_) => self.active_view = Views::Main,
            },
            Message::AddWithTitle(title) => {
                self.append_to_config(".*".to_string(), title);
            }
            Message::AddWithAPPID(id) => {
                self.append_to_config(id, "".to_string());
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl App {
    fn list_view(&self) -> Element<'_, Message> {
        let mut content_list = widget::settings::section().add(
            widget::settings::item_row(vec![
                widget::text(fl!("app-title")).into(),
                widget::button::suggested(fl!("refresh"))
                    .on_press(Message::RefreshToplevels)
                    .into(),
            ])
            .spacing(12),
        );

        for (idx, app) in self.apps_info.iter().enumerate() {
            content_list = content_list.add(widget::settings::item_row(vec![
                widget::button::standard(fl!("more"))
                    .on_press(Message::ChangeView(idx))
                    .into(),
                widget::text(app.title.clone()).into(),
            ]));
        }

        widget::column()
            .push(padded_control(widget::Container::new(Scrollable::new(
                content_list,
            ))))
            .into()
    }

    fn manage_view(&self, index: usize) -> Element<'_, Message> {
        let toplevel = &self.apps_info[index];

        let controls = widget::row()
            .spacing(12)
            .padding(12)
            .push(widget::button::standard(fl!("back")).on_press(Message::BackToMain))
            .push(
                widget::button::standard(fl!("add-with-title"))
                    .on_press(Message::AddWithTitle(toplevel.title.clone())),
            )
            .push(
                widget::button::standard(fl!("add-with-appid"))
                    .on_press(Message::AddWithAPPID(toplevel.app_id.clone())),
            );

        let col = widget::column()
            .spacing(12)
            .padding(12)
            .push(widget::text::title4(fl!("title")))
            .push(widget::text(toplevel.title.clone()))
            .push(widget::text::title4(fl!("appid")))
            .push(widget::text(toplevel.app_id.clone()));

        widget::list_column()
            .padding(5)
            .spacing(5)
            .add(controls)
            .add(col)
            .into()
    }

    fn append_to_config(&self, appid: String, title: String) {
        let window_rules_context =
            window_rules::context().expect("Failed to load window rules config");

        let mut rules = window_rules_context
            .get::<Vec<window_rules::PreciseApplicationException>>("tiling_exception_custom")
            .unwrap_or_else(|why| {
                if why.is_err()
                    && let cosmic::cosmic_config::Error::GetKey(_, err) = &why
                    && err.kind() != std::io::ErrorKind::NotFound
                {
                    eprintln!("tiling exceptions custom config error: {why}");
                    return Vec::new();
                }
                eprintln!("tiling exceptions custom config not present: {why}");
                Vec::new()
            });

        let new_exception = window_rules::PreciseApplicationException {
            appid,
            title,
            enabled: true,
        };

        rules.push(new_exception);

        window_rules_context
            .set("tiling_exception_custom", rules)
            .expect("cannot write config")
    }
}
