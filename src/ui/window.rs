use adw::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::app::APP_NAME;
use crate::backend;
use libadwaita as adw;
use std::time::Duration;

const DEFAULT_WIDTH: i32 = 840;
const DEFAULT_HEIGHT: i32 = 540;
const MIN_WIDTH: i32 = 420;
const MIN_HEIGHT: i32 = 140;
const SIDEBAR_COLLAPSE_WIDTH: f64 = 620.0;

use super::{
    AboutPage, AuraPage, Page, PowerPage, PreferencesDialog, Refreshable, SlashPage, ThemeSwitcher,
};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct AsusctlGuiWindow {
        pub split_view: RefCell<Option<adw::NavigationSplitView>>,
        pub stack: RefCell<Option<gtk4::Stack>>,
        pub sidebar: RefCell<Option<adw::Sidebar>>,
        pub settings: RefCell<Option<gio::Settings>>,
        // Store direct references to pages for refresh
        pub about_page: RefCell<Option<AboutPage>>,
        pub aura_page: RefCell<Option<AuraPage>>,
        pub power_page: RefCell<Option<PowerPage>>,
        pub slash_page: RefCell<Option<SlashPage>>,
        // Track refresh timer source ID
        pub refresh_source_id: RefCell<Option<glib::SourceId>>,
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
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("title", APP_NAME)
            .property("default-width", DEFAULT_WIDTH)
            .property("default-height", DEFAULT_HEIGHT)
            .build();
        window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
        window
    }

    /// Start a periodic timer that refreshes the visible page
    fn start_refresh_timer(&self, interval_secs: f64) {
        let imp = self.imp();
        let window_weak = self.downgrade();
        let millis = (interval_secs * 1000.0) as u64;

        let source_id = glib::timeout_add_local(Duration::from_millis(millis), move || {
            let Some(window) = window_weak.upgrade() else {
                return glib::ControlFlow::Break;
            };

            window.refresh_visible_page();
            glib::ControlFlow::Continue
        });

        imp.refresh_source_id.replace(Some(source_id));
    }

    /// Restart the refresh timer with new interval
    fn restart_refresh_timer(&self, interval_secs: f64) {
        let imp = self.imp();

        // Stop existing timer
        if let Some(source_id) = imp.refresh_source_id.take() {
            source_id.remove();
        }

        // Start new timer
        self.start_refresh_timer(interval_secs);
    }

    /// Refresh the currently visible page
    fn refresh_visible_page(&self) {
        let imp = self.imp();

        let Some(stack) = imp.stack.borrow().as_ref().cloned() else {
            return;
        };

        let Some(name) = stack.visible_child_name() else {
            return;
        };

        let Ok(page) = Page::try_from(name.as_str()) else {
            return;
        };

        match page {
            Page::About => {
                if let Some(p) = imp.about_page.borrow().as_ref() {
                    p.refresh();
                }
            }
            Page::Aura => {
                if let Some(p) = imp.aura_page.borrow().as_ref() {
                    p.refresh();
                }
            }
            Page::Power => {
                if let Some(p) = imp.power_page.borrow().as_ref() {
                    p.refresh();
                }
            }
            Page::Slash => {
                if let Some(p) = imp.slash_page.borrow().as_ref() {
                    p.refresh();
                }
            }
        }
    }

    fn setup_ui(&self) {
        let imp = self.imp();
        let settings = gio::Settings::new("com.github.bl4ckspell7.asusctl-gui");

        // Create the content stack for pages
        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::None)
            .hhomogeneous(false)
            .vhomogeneous(false)
            .build();

        // Create sidebar with navigation items using adw::Sidebar
        let sidebar = adw::Sidebar::new();
        let section = adw::SidebarSection::new();

        for page in Page::ALL {
            let item = adw::SidebarItem::builder()
                .title(page.title())
                .icon_name(page.icon())
                .build();
            section.append(item);
        }

        sidebar.append(section);

        // Connect sidebar activation to stack page switching
        let stack_clone = stack.clone();
        let settings_clone = settings.clone();
        sidebar.connect_activated(move |sidebar, index| {
            if let Some(page) = Page::from_index(index) {
                stack_clone.set_visible_child_name(page.as_str());
                let _ = settings_clone.set_string("last-page", page.as_str());

                // When collapsed, navigate to the content pane
                if let Some(split_view) = sidebar
                    .ancestor(adw::NavigationSplitView::static_type())
                    .and_then(|w| w.downcast::<adw::NavigationSplitView>().ok())
                {
                    split_view.set_show_content(true);
                }
            }
        });

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
        buttons_section.append(Some("About"), Some("win.about"));
        menu.append_section(None, &buttons_section);

        // Quit section
        let quit_section = gio::Menu::new();
        quit_section.append(Some("Quit"), Some("win.quit"));
        menu.append_section(None, &quit_section);

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
            .title_widget(&gtk4::Label::new(Some(APP_NAME)))
            .build();
        sidebar_header.pack_end(&menu_button);

        let sidebar_toolbar = adw::ToolbarView::new();
        sidebar_toolbar.add_top_bar(&sidebar_header);
        sidebar_toolbar.set_content(Some(&sidebar));

        // Create sidebar navigation page
        let sidebar_page = adw::NavigationPage::builder()
            .title("asusctl")
            .child(&sidebar_toolbar)
            .build();

        // Create content toolbar view with header
        let content_header = adw::HeaderBar::builder().show_title(false).build();

        let content_toolbar = adw::ToolbarView::new();
        content_toolbar.add_top_bar(&content_header);
        content_toolbar.set_content(Some(&stack));

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

        // Collapse sidebar on narrow windows and switch to boxed list mode
        let breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
            adw::BreakpointConditionLengthType::MaxWidth,
            SIDEBAR_COLLAPSE_WIDTH,
            adw::LengthUnit::Sp,
        ));
        breakpoint.add_setter(&split_view, "collapsed", Some(&true.to_value()));
        breakpoint.add_setter(&sidebar, "mode", Some(&adw::SidebarMode::Page.to_value()));
        self.add_breakpoint(breakpoint);

        // Show a loading screen while features are detected in the background
        let loading_header = adw::HeaderBar::builder()
            .title_widget(&gtk4::Label::new(Some(APP_NAME)))
            .build();
        let spinner = adw::Spinner::builder()
            .width_request(64)
            .height_request(64)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .vexpand(true)
            .build();
        let loading_view = adw::ToolbarView::new();
        loading_view.add_top_bar(&loading_header);
        loading_view.set_content(Some(&spinner));
        self.set_content(Some(&loading_view));

        // Setup actions
        self.setup_actions();

        // Store references (split view swapped in once loading finishes)
        imp.split_view.replace(Some(split_view));
        imp.stack.replace(Some(stack));
        imp.sidebar.replace(Some(sidebar));
        imp.settings.replace(Some(settings.clone()));

        // Start refresh timer with interval from settings (in seconds)
        let interval_secs = settings.double("refresh-interval");
        self.start_refresh_timer(interval_secs);

        // Listen for settings changes to restart timer with new interval
        let window_weak = self.downgrade();
        settings.connect_changed(Some("refresh-interval"), move |settings, _| {
            if let Some(window) = window_weak.upgrade() {
                let new_interval = settings.double("refresh-interval");
                window.restart_refresh_timer(new_interval);
            }
        });

        self.start_background_detection();
    }

    /// Run feature detection on a background thread, then populate pages on the main thread.
    fn start_background_detection(&self) {
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let ready_thread = ready.clone();
        std::thread::spawn(move || {
            backend::detect_features();
            ready_thread.store(true, std::sync::atomic::Ordering::Release);
        });
        let window_weak = self.downgrade();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            if ready.load(std::sync::atomic::Ordering::Acquire) {
                if let Some(window) = window_weak.upgrade() {
                    window.populate_pages();
                }
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    /// Called once feature detection completes on the background thread.
    /// Creates the real page widgets and swaps out the loading spinner.
    fn populate_pages(&self) {
        let imp = self.imp();

        let Some(stack) = imp.stack.borrow().as_ref().cloned() else {
            return;
        };
        let Some(settings) = imp.settings.borrow().as_ref().cloned() else {
            return;
        };

        // Create pages (detect_features() is now cached and returns instantly)
        let about_page = AboutPage::new();
        let aura_page = AuraPage::new();
        let power_page = PowerPage::new();
        let slash_page = SlashPage::new();

        // Wrap each page in its own ScrolledWindow so scroll state is independent
        for (page_widget, page) in [
            (about_page.upcast_ref::<gtk4::Widget>(), Page::About),
            (aura_page.upcast_ref::<gtk4::Widget>(), Page::Aura),
            (power_page.upcast_ref::<gtk4::Widget>(), Page::Power),
            (slash_page.upcast_ref::<gtk4::Widget>(), Page::Slash),
        ] {
            let scroll = gtk4::ScrolledWindow::builder()
                .hscrollbar_policy(gtk4::PolicyType::Never)
                .vscrollbar_policy(gtk4::PolicyType::Automatic)
                .child(page_widget)
                .build();
            stack.add_titled(&scroll, Some(page.as_str()), page.title());
        }

        // Store page references for later refresh
        imp.about_page.replace(Some(about_page));
        imp.aura_page.replace(Some(aura_page));
        imp.power_page.replace(Some(power_page));
        imp.slash_page.replace(Some(slash_page));

        // Swap loading screen for the full split view
        if let Some(split_view) = imp.split_view.borrow().as_ref() {
            self.set_content(Some(split_view));
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

        // Select corresponding sidebar item
        if let Some(sidebar) = imp.sidebar.borrow().as_ref() {
            sidebar.set_selected(startup_page.index());
        }
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
            .application_name(APP_NAME)
            .application_icon("com.github.bl4ckspell7.asusctl-gui")
            .developer_name("Bl4ckspell")
            .version(env!("CARGO_PKG_VERSION"))
            .website("https://github.com/Bl4ckspell7/asusctl-gui")
            .license_type(gtk4::License::Gpl30)
            .build();

        about.present(Some(self));
    }

    fn show_shortcuts_dialog(&self) {
        let shortcuts = adw::ShortcutsDialog::new();

        // Create section with items
        let section = adw::ShortcutsSection::new(Some("General"));
        section.add(adw::ShortcutsItem::new("Preferences", "<Control>comma"));
        section.add(adw::ShortcutsItem::new(
            "Keyboard Shortcuts",
            "<Control>question",
        ));
        section.add(adw::ShortcutsItem::new("About", "F1"));
        section.add(adw::ShortcutsItem::new("Quit", "<Control>q"));

        shortcuts.add(section);
        shortcuts.present(Some(self));
    }
}
