// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::iced::{Length, Subscription};
use cosmic::widget;
use cosmic::{Application, Element};
use niri_ipc::Workspace;
use std::sync::mpsc;

use super::niri;

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct WorkspacesApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    workspaces: Vec<Workspace>,
    sender: Option<mpsc::Sender<u64>>,
}
impl WorkspacesApp {
    pub fn new(core: Core) -> Self {
        Self {
            core,
            workspaces: Vec::new(),
            sender: None,
        }
    }
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    Ready(mpsc::Sender<u64>),
    WorkspaceUpdate(Vec<Workspace>),
    WorkspaceActivated { id: u64, focused: bool },
    ActivateWorkspace(u64),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for WorkspacesApp {
    type Executor = cosmic::SingleThreadExecutor;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.niri.workspaces";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = WorkspacesApp::new(core);

        (app, Task::none())
    }

    // fn on_close_requested(&self, id: Id) -> Option<Message> {
    //     Some(Message::PopupClosed(id))
    // }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let horizontal = matches!(
            self.core.applet.anchor,
            PanelAnchor::Top | PanelAnchor::Bottom
        );
        let mut children: Vec<Element<Message>> = Vec::with_capacity(self.workspaces.len());
        for workspace in &self.workspaces {
            let class = match workspace.is_active {
                true => cosmic::style::Button::Suggested,
                false => cosmic::style::Button::Standard,
            };
            let height = if horizontal {
                Length::Fixed(self.core.applet.suggested_size(false).1 as f32)
            } else {
                Length::Fixed(16.0)
            };
            let width = if !horizontal {
                Length::Fixed(self.core.applet.suggested_size(false).1 as f32)
            } else {
                Length::Fixed(16.0)
            };
            children.push(
                widget::button::custom(cosmic::widget::Space::new(width, height))
                    .class(class)
                    .on_press(Message::ActivateWorkspace(workspace.id))
                    .into(),
            )
        }
        let container: Element<Message> = if !horizontal {
            widget::Column::with_children(children)
                .spacing(4)
                .padding(8)
                .into()
        } else {
            widget::Row::with_children(children)
                .spacing(4)
                .padding(8)
                .into()
        };
        self.core.applet.autosize_window(container).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::WorkspaceUpdate(mut workspaces) => {
                workspaces.sort_unstable_by_key(|a| a.id);
                self.workspaces = workspaces;
            }
            Message::WorkspaceActivated { id, focused } => {
                for workspace in self.workspaces.iter_mut() {
                    if workspace.id == id {
                        workspace.is_active = true;
                        workspace.is_focused = focused;
                    } else {
                        workspace.is_active = false;
                        workspace.is_focused = false;
                    }
                }
            }
            Message::ActivateWorkspace(id) => {
                for workspace in self.workspaces.iter_mut() {
                    if workspace.id == id {
                        workspace.is_active = true;
                    } else {
                        workspace.is_active = false;
                    }
                }
                if let Some(sender) = &self.sender {
                    sender.send(id).unwrap();
                }
            }
            Message::Ready(sender) => self.sender = Some(sender),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(niri::sub)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
