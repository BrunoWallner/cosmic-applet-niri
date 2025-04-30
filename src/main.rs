// SPDX-License-Identifier: GPL-3.0-only
mod workspaces;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<workspaces::WorkspacesApp>(())
}
