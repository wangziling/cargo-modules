// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use yansi::Style;

use crate::colors::cli::color_palette;

#[derive(Clone, Debug)]
pub(crate) struct Styles {
    pub chrome: Style,
    pub insertion: Style,
    pub deletion: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub help: Style,
}

pub(crate) fn styles() -> Styles {
    let color_palette = color_palette();
    Styles {
        chrome: Style::new(color_palette.blue).bold(),
        insertion: Style::new(color_palette.green).bold(),
        deletion: Style::new(color_palette.red).bold(),
        success: Style::new(color_palette.green).bold(),
        error: Style::new(color_palette.red).bold(),
        warning: Style::new(color_palette.orange).bold(),
        help: Style::new(color_palette.cyan).bold(),
    }
}
