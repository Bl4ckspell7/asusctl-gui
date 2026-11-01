mod pages;
mod preferences_dialog;
mod theme_switcher;
mod toast;
mod window;

pub use pages::{AboutPage, AuraPage, GpuPage, PowerPage, SlashPage};
pub use preferences_dialog::PreferencesDialog;
pub use theme_switcher::ThemeSwitcher;
pub use window::AsusctlGuiWindow;

pub(crate) use toast::show_backend_error;

use std::fmt;

/// Trait for pages that can refresh their data
pub trait Refreshable {
    fn refresh(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    About,
    Aura,
    Power,
    Gpu,
    Slash,
}

impl Page {
    pub const ALL: [Page; 5] = [Page::About, Page::Aura, Page::Power, Page::Gpu, Page::Slash];

    pub fn as_str(&self) -> &'static str {
        match self {
            Page::About => "about",
            Page::Aura => "aura",
            Page::Power => "power",
            Page::Gpu => "gpu",
            Page::Slash => "slash",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Page::About => "About",
            Page::Aura => "Aura",
            Page::Power => "Power",
            Page::Gpu => "GPU",
            Page::Slash => "Slash",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Page::About => "computer-symbolic",
            Page::Aura => "keyboard-brightness-symbolic",
            Page::Power => "gnome-power-manager-symbolic",
            Page::Gpu => "video-display-symbolic",
            Page::Slash => "display-brightness-symbolic",
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            Page::About => 0,
            Page::Aura => 1,
            Page::Power => 2,
            Page::Gpu => 3,
            Page::Slash => 4,
        }
    }

    pub fn from_index(index: u32) -> Option<Page> {
        match index {
            0 => Some(Page::About),
            1 => Some(Page::Aura),
            2 => Some(Page::Power),
            3 => Some(Page::Gpu),
            4 => Some(Page::Slash),
            _ => None,
        }
    }
}

impl TryFrom<&str> for Page {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "about" => Ok(Page::About),
            "aura" => Ok(Page::Aura),
            "power" => Ok(Page::Power),
            "gpu" => Ok(Page::Gpu),
            "slash" => Ok(Page::Slash),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
