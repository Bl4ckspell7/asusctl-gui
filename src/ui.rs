mod pages;
mod preferences_dialog;
mod theme_switcher;
mod toast;
mod window;

pub use pages::{AboutPage, AuraPage, GpuPage, PowerPage, SlashPage, SystemPage};
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
    System,
}

impl Page {
    pub const ALL: [Page; 6] = [
        Page::About,
        Page::Aura,
        Page::Power,
        Page::Gpu,
        Page::Slash,
        Page::System,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Page::About => "about",
            Page::Aura => "aura",
            Page::Power => "power",
            Page::Gpu => "gpu",
            Page::Slash => "slash",
            Page::System => "system",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Page::About => "About",
            Page::Aura => "Aura",
            Page::Power => "Power",
            Page::Gpu => "GPU",
            Page::Slash => "Slash",
            Page::System => "System",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Page::About => "computer-symbolic",
            Page::Aura => "keyboard-brightness-symbolic",
            Page::Power => "gnome-power-manager-symbolic",
            Page::Gpu => "video-display-symbolic",
            Page::Slash => "display-brightness-symbolic",
            Page::System => "emblem-system-symbolic",
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            Page::About => 0,
            Page::Aura => 1,
            Page::Power => 2,
            Page::Gpu => 3,
            Page::Slash => 4,
            Page::System => 5,
        }
    }

    pub fn from_index(index: u32) -> Option<Page> {
        match index {
            0 => Some(Page::About),
            1 => Some(Page::Aura),
            2 => Some(Page::Power),
            3 => Some(Page::Gpu),
            4 => Some(Page::Slash),
            5 => Some(Page::System),
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
            "system" => Ok(Page::System),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_lookups_agree_with_all() {
        for (position, page) in Page::ALL.iter().enumerate() {
            assert_eq!(page.index() as usize, position, "{page} index");
            assert_eq!(
                Page::from_index(page.index()),
                Some(*page),
                "{page} from_index"
            );
            assert_eq!(Page::try_from(page.as_str()), Ok(*page), "{page} try_from");
        }
        assert_eq!(Page::from_index(Page::ALL.len() as u32), None);
        assert_eq!(Page::try_from("unknown"), Err(()));
    }
}
