// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Terminal utilities.

use termion;
use std;
use std::io::stdout;
use std::fmt;

/// Get the number of columns for the terminal from `$COLUMNS`.
///
/// Return `None` if the variable is not set or does not contain a valid number.
fn columns_from_env() -> Option<u16> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
}

/// Get the number of columns from the TTY device.
///
/// Return `None` if TTY access fails.
fn columns_from_tty() -> Option<u16> {
    termion::terminal_size().map(|size| size.0).ok()
}

/// Make a best effort to get the number of columns for the terminal.
///
/// Try to get the terminal size from the TTY device, or from the `$COLUMNS`
/// environment variable, and eventually assume a default of 80 for safety.
pub fn columns() -> u16 {
    columns_from_tty().or_else(columns_from_env).unwrap_or(80)
}

/// The terminal we use.
#[derive(Debug)]
enum Terminal {
    /// iTerm2.
    ///
    /// iTerm2 is a powerful macOS terminal emulator with many formatting
    /// features, including images and inline links.
    ///
    /// See <https://www.iterm2.com> for more information.
    ITerm2,
    /// A terminal based on a modern VTE version.
    ///
    /// VTE is Gnome library for terminal emulators.  It powers some notable
    /// terminal emulators like Gnome Terminal, and embedded terminals in
    /// applications like GEdit.
    ///
    /// We require 0.50 or newer; these versions support inline links.
    VTE50,
    /// An unknown terminal application.
    Unknown,
}

/// Get the version of VTE underlying this terminal.
///
/// Return `(minor, patch)` if this terminal uses VTE, otherwise return `None`.
fn get_vte_version() -> Option<(u8, u8)> {
    std::env::var("VTE_VERSION").ok().and_then(|value| {
        value[..2]
            .parse::<u8>()
            .into_iter()
            .zip(value[2..4].parse::<u8>())
            .next()
    })
}

impl Terminal {
    /// Detect the underlying terminal application.
    fn detect() -> Terminal {
        if std::env::var("TERM_PROGRAM")
            .map(|value| value.contains("iTerm.app"))
            .unwrap_or(false)
        {
            Terminal::ITerm2
        } else {
            match get_vte_version() {
                Some(version) if version >= (50, 0) => Terminal::VTE50,
                _ => Terminal::Unknown,
            }
        }
    }
}

#[derive(Debug)]
pub struct Format {
    /// Whether to enable basic colours.
    basic_colours: bool,
    /// Whether to enable inline links.
    inline_links: bool,
    /// Whether to render images inline.
    inline_images: bool,
    /// Whether to set iterm marks for headings.
    iterm_marks: bool,
}

impl Format {
    /// Create an empty format.
    ///
    /// This format enables no special formatting.
    pub fn empty() -> Format {
        Format {
            basic_colours: false,
            inline_links: false,
            inline_images: false,
            iterm_marks: false,
        }
    }

    /// Auto-detect the format to use.
    ///
    /// If `force_colours` is true enforce colours, otherwise use colours if we run
    /// on a TTY.  If we run on a TTY and detect that we run within iTerm, enable
    /// additional formatting for iTerm.
    pub fn auto_detect(force_colours: bool) -> Format {
        let mut format = Format::empty();
        if termion::is_tty(&stdout()) {
            format.basic_colours = true;
            match Terminal::detect() {
                Terminal::ITerm2 => Format {
                    basic_colours: true,
                    inline_links: true,
                    inline_images: true,
                    iterm_marks: true,
                },
                Terminal::VTE50 => Format {
                    basic_colours: true,
                    inline_links: true,
                    ..Format::empty()
                },
                Terminal::Unknown => Format {
                    basic_colours: true,
                    ..Format::empty()
                },
            }
        } else {
            Format {
                basic_colours: force_colours,
                ..Format::empty()
            }
        }
    }

    /// Whether this format enables colours.
    pub fn enables_colours(&self) -> bool {
        self.basic_colours
    }

    /// Whether this format enables inline links.
    pub fn enables_inline_links(&self) -> bool {
        self.inline_links
    }

    /// Whether this format enables inline images.
    pub fn enables_inline_images(&self) -> bool {
        self.inline_images
    }

    /// Whether this format enables marks.
    pub fn enables_iterm_marks(&self) -> bool {
        self.iterm_marks
    }
}

/// An OSC command for a terminal.
#[derive(Debug, Copy, Clone)]
pub struct OSC<'a> {
    command: &'a str,
}

/// Create an OSC command for the terminal.
pub fn osc(command: &str) -> OSC {
    OSC { command }
}

impl<'a> fmt::Display for OSC<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B]{}\x07", self.command)
    }
}
