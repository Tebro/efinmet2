// SPDX-License-Identifier: {{LICENSE}}

use crate::fl;
use crate::metars;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Padding;
use cosmic::iced::{Alignment, Length};
use cosmic::widget;
use cosmic::{Application, ApplicationExt, Apply, Element};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,

    metars: Vec<String>,
    error: Option<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LoadMetars,
    UpdateMetars(Vec<String>),
    SetError(String),
}

/// Create a COSMIC application from the app model
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "org.tebro.efinmet2";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            metars: Vec::new(),
            error: None,
        };

        // Create a startup command that sets the window title.
        let title_cmd = app.update_title();
        let loop_started_cmd =
            cosmic::command::message(cosmic::app::Message::App(Message::LoadMetars));

        let command = cosmic::command::batch(vec![title_cmd, loop_started_cmd]);

        (app, command)
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        if let Some(error) = &self.error {
            return widget::text::Text::new(error)
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into();
        }

        let metar_texts = self.metars.iter().map(|metar| -> Element<Self::Message> {
            widget::text::Text::new(metar.strip_prefix("METAR ").unwrap_or(metar))
                .apply(widget::container)
                .width(Length::Fill)
                //.height(Length::Fill)
                .align_x(Horizontal::Left)
                .padding(Padding {
                    left: 10.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                })
                .align_y(Vertical::Center)
                .into()
        });

        let mut col = widget::column();

        col = col.push(
            widget::text::Text::new("EFIN Metars")
                .apply(widget::container)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .padding(Padding {
                    left: 0.,
                    right: 0.0,
                    top: 0.0,
                    bottom: 20.0,
                })
                .align_y(Vertical::Center),
        );

        for metar_text in metar_texts {
            col = col.push(metar_text);
        }

        col.align_items(Alignment::Center).into()
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Commands may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LoadMetars => {
                return cosmic::command::future(async move {
                    let metars = metars::fetch_metars().await;

                    match metars {
                        Ok(metars) => {
                            return cosmic::app::Message::App(Message::UpdateMetars(metars));
                        }
                        Err(error) => {
                            return cosmic::app::Message::App(Message::SetError(error.to_string()));
                        }
                    }
                });
            }

            Message::UpdateMetars(metars) => {
                self.metars = metars;
                self.error = None;

                // Ugly hack loop?
                return cosmic::command::future(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    cosmic::app::Message::App(Message::LoadMetars)
                });
            }

            Message::SetError(error) => {
                let full_error = format!("Error (will retry shortly): {}", error);
                self.error = Some(full_error);
            }
        }
        Command::none()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Command<Message> {
        let window_title = fl!("app-title");

        self.set_window_title(window_title)
    }
}
