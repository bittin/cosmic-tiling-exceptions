// SPDX-License-Identifier: GPL-3.0-only

use app::App;
mod app;
mod core;

pub(crate) mod wayland;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<App>(())
}
