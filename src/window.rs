use adw::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;

use crate::page::Page;
use crate::pages::{AboutPage, AuraPage, ProfilePage, SlashPage};
use crate::preferences_dialog::PreferencesDialog;
use crate::theme_switcher::ThemeSwitcher;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct AsusctlGuiWindow {
        pub split_view: RefCell<Option<adw::NavigationSplitView>>,
        pub stack: RefCell<Option<gtk4::Stack>>,
        pub sidebar_list: RefCell<Option<gtk4::ListBox>>,
        pub settings: RefCell<Option<gio::Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AsusctlGuiWindow {
        const NAME: &'static str = "AsusctlGuiWindow";
        type Type = super::AsusctlGuiWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for AsusctlGuiWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for AsusctlGuiWindow {}
    impl WindowImpl for AsusctlGuiWindow {}
    impl ApplicationWindowImpl for AsusctlGuiWindow {}
    impl AdwApplicationWindowImpl for AsusctlGuiWindow {}
}

glib::wrapper! {
    pub struct AsusctlGuiWindow(ObjectSubclass<imp::AsusctlGuiWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget,
                    gtk4::Native, gtk4::Root, gtk4::ShortcutManager,
                    gio::ActionGroup, gio::ActionMap;
}

impl AsusctlGuiWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .property("title", "asusctl-gui")
            .property("default-width", 840)
            .property("default-height", 540)
            .build()
    }

    fn setup_ui(&self) {
        let settings = gio::Settings::new("com.github.bl4ckspell7.asusctl-gui");

        // Create the content stack for pages
        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .hhomogeneous(false)
            .vhomogeneous(false)
            .build();

        // Add pages to stack
        let about_page = AboutPage::new();
        let aura_page = AuraPage::new();
        let profile_page = ProfilePage::new();
        let slash_page = SlashPage::new();

        stack.add_titled(&about_page, Some(Page::About.as_str()), Page::About.title());
        stack.add_titled(&aura_page, Some(Page::Aura.as_str()), Page::Aura.title());
        stack.add_titled(
            &profile_page,
            Some(Page::Profile.as_str()),
            Page::Profile.title(),
        );
        stack.add_titled(&slash_page, Some(Page::Slash.as_str()), Page::Slash.title());

        // Create sidebar with navigation items
        let sidebar_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();

        // Add navigation rows using Page enum
        for page in Page::ALL {
            let row = Self::create_nav_row(page);
            sidebar_list.append(&row);
        }

        // Determine startup page
        let startup_page = if settings.boolean("restore-last-page") {
            let last_page_str = settings.string("last-page");
            Page::try_from(last_page_str.as_str()).unwrap_or_default()
        } else {
            let startup_page_str = settings.string("startup-page");
            Page::try_from(startup_page_str.as_str()).unwrap_or_default()
        };

        // Set initial page
        stack.set_visible_child_name(startup_page.as_str());

        // Select corresponding sidebar row
        if let Some(row) = sidebar_list.row_at_index(startup_page.index() as i32) {
            sidebar_list.select_row(Some(&row));
        }

        // Connect row selection to stack page switching
        let stack_clone = stack.clone();
        let settings_clone = settings.clone();
        sidebar_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(name) = row.widget_name().as_str().strip_prefix("nav-") {
                    stack_clone.set_visible_child_name(name);
                    // Save last viewed page
                    let _ = settings_clone.set_string("last-page", name);
                }
            }
        });

        // Wrap sidebar in a scrolled window
        let sidebar_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&sidebar_list)
            .build();

        // Create hamburger menu
        let menu = gio::Menu::new();

        // Theme section (custom widget placeholder)
        let theme_section = gio::Menu::new();
        let theme_item = gio::MenuItem::new(None, None);
        theme_item.set_attribute_value("custom", Some(&"themeswitcher".to_variant()));
        theme_section.append_item(&theme_item);
        menu.append_section(None, &theme_section);

        // Buttons section
        let buttons_section = gio::Menu::new();
        buttons_section.append(Some("Preferences"), Some("win.preferences"));
        buttons_section.append(Some("Keyboard Shortcuts"), Some("win.show-shortcuts"));
        buttons_section.append(Some("Quit"), Some("win.quit"));
        buttons_section.append(Some("About"), Some("win.about"));
        menu.append_section(None, &buttons_section);

        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .primary(true)
            .tooltip_text("Main Menu")
            .build();

        // Add ThemeSwitcher as custom child to the popover
        if let Some(popover) = menu_button.popover() {
            if let Ok(popover_menu) = popover.downcast::<gtk4::PopoverMenu>() {
                popover_menu.add_child(&ThemeSwitcher::new(), "themeswitcher");
            }
        }

        // Create sidebar toolbar view with header
        let sidebar_header = adw::HeaderBar::builder()
            .title_widget(&gtk4::Label::new(Some("asusctl-gui")))
            .build();
        sidebar_header.pack_end(&menu_button);

        let sidebar_toolbar = adw::ToolbarView::new();
        sidebar_toolbar.add_top_bar(&sidebar_header);
        sidebar_toolbar.set_content(Some(&sidebar_scroll));

        // Create sidebar navigation page
        let sidebar_page = adw::NavigationPage::builder()
            .title("asusctl")
            .child(&sidebar_toolbar)
            .build();

        // Create content toolbar view with header (no menu button here anymore)
        let content_header = adw::HeaderBar::builder().show_title(false).build();

        // Wrap stack in a scrolled window to allow content scrolling
        let content_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&stack)
            .build();

        let content_toolbar = adw::ToolbarView::new();
        content_toolbar.add_top_bar(&content_header);
        content_toolbar.set_content(Some(&content_scroll));

        // Create content navigation page
        let content_page = adw::NavigationPage::builder()
            .title("Content")
            .child(&content_toolbar)
            .build();

        // Create split view
        let split_view = adw::NavigationSplitView::builder()
            .sidebar(&sidebar_page)
            .content(&content_page)
            .min_sidebar_width(200.0)
            .max_sidebar_width(300.0)
            .build();

        self.set_content(Some(&split_view));

        // Setup actions
        self.setup_actions();

        // Store references
        let imp = self.imp();
        imp.split_view.replace(Some(split_view));
        imp.stack.replace(Some(stack));
        imp.sidebar_list.replace(Some(sidebar_list));
        imp.settings.replace(Some(settings));
    }

    fn setup_actions(&self) {
        // Preferences action
        let preferences_action = gio::SimpleAction::new("preferences", None);
        let window = self.clone();
        preferences_action.connect_activate(move |_, _| {
            window.show_preferences_dialog();
        });
        self.add_action(&preferences_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        let window = self.clone();
        about_action.connect_activate(move |_, _| {
            window.show_about_dialog();
        });
        self.add_action(&about_action);

        // Shortcuts action
        let shortcuts_action = gio::SimpleAction::new("show-shortcuts", None);
        let window = self.clone();
        shortcuts_action.connect_activate(move |_, _| {
            window.show_shortcuts_dialog();
        });
        self.add_action(&shortcuts_action);

        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        let window = self.clone();
        quit_action.connect_activate(move |_, _| {
            window.close();
        });
        self.add_action(&quit_action);
    }

    fn show_preferences_dialog(&self) {
        let prefs_dialog = PreferencesDialog::new();
        prefs_dialog.present(Some(self));
    }

    fn show_about_dialog(&self) {
        let about = adw::AboutDialog::builder()
            .application_name("asusctl-gui")
            .application_icon("preferences-other-symbolic")
            .developer_name("Bl4ckspell")
            .version("0.1.0")
            .website("https://github.com/Bl4ckspell7/asusctl-gui")
            .license_type(gtk4::License::Gpl30)
            .build();

        about.present(Some(self));
    }

    fn show_shortcuts_dialog(&self) {
        let shortcuts = adw::ShortcutsDialog::new();

        // Create section with items
        let section = adw::ShortcutsSection::new(Some("General"));
        section.add(adw::ShortcutsItem::new("Quit", "<Control>q"));
        section.add(adw::ShortcutsItem::new(
            "Keyboard Shortcuts",
            "<Control>question",
        ));

        shortcuts.add(section);
        shortcuts.present(Some(self));
    }

    fn create_nav_row(page: Page) -> gtk4::ListBoxRow {
        let hbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let icon = gtk4::Image::from_icon_name(page.icon());
        let label = gtk4::Label::builder()
            .label(page.title())
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        hbox.append(&icon);
        hbox.append(&label);

        gtk4::ListBoxRow::builder()
            .child(&hbox)
            .name(format!("nav-{}", page.as_str()))
            .build()
    }
}
