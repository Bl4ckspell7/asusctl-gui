use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    About,
    Aura,
    Profile,
    Slash,
}

impl Page {
    pub const ALL: [Page; 4] = [Page::About, Page::Aura, Page::Profile, Page::Slash];

    pub fn as_str(&self) -> &'static str {
        match self {
            Page::About => "about",
            Page::Aura => "aura",
            Page::Profile => "profile",
            Page::Slash => "slash",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Page::About => "About",
            Page::Aura => "Aura",
            Page::Profile => "Profile",
            Page::Slash => "Slash",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Page::About => "computer-symbolic",
            Page::Aura => "keyboard-brightness-symbolic",
            Page::Profile => "power-profile-balanced-symbolic",
            Page::Slash => "display-brightness-symbolic",
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            Page::About => 0,
            Page::Aura => 1,
            Page::Profile => 2,
            Page::Slash => 3,
        }
    }

    pub fn from_index(index: u32) -> Option<Page> {
        match index {
            0 => Some(Page::About),
            1 => Some(Page::Aura),
            2 => Some(Page::Profile),
            3 => Some(Page::Slash),
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
            "profile" => Ok(Page::Profile),
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
