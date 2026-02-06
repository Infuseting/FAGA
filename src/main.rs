use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, Row, Space,
};
use iced::{Alignment, Color, Element, Length, Padding, Settings, Theme, Application, Command, Font, Subscription};
use iced::window;
use iced::mouse;
use iced::event::{self, Event};
use iced::keyboard;

mod network;
mod parser;

use network::HttpClient;
use parser::{HtmlParser, HtmlRenderer, flatten_render_tree_with_body, StyledText};

/// Résout une URL relative par rapport à une URL de base
fn resolve_url(base_url: &str, href: &str) -> String {
    // Si l'URL est déjà absolue, la retourner telle quelle
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }

    // Si c'est une URL de protocole relatif
    if href.starts_with("//") {
        let protocol = if base_url.starts_with("https://") { "https:" } else { "http:" };
        return format!("{}{}", protocol, href);
    }

    // Parser l'URL de base
    let base = if let Some(pos) = base_url.find("://") {
        let protocol = &base_url[..pos + 3];
        let rest = &base_url[pos + 3..];

        // Trouver l'origine (host + port)
        let origin_end = rest.find('/').unwrap_or(rest.len());
        let origin = &rest[..origin_end];
        let base_path = &rest[origin_end..];

        (protocol.to_string(), origin.to_string(), base_path.to_string())
    } else {
        return href.to_string(); // URL de base invalide
    };

    let (protocol, origin, base_path) = base;

    // URL absolue par rapport à l'origine
    if href.starts_with('/') {
        return format!("{}{}{}", protocol, origin, href);
    }

    // URL relative par rapport au chemin actuel
    // Retirer le dernier segment du chemin de base
    let parent_path = if let Some(pos) = base_path.rfind('/') {
        &base_path[..pos + 1]
    } else {
        "/"
    };

    format!("{}{}{}{}", protocol, origin, parent_path, href)
}

// Police avec support Unicode étendu (cross-platform)
#[cfg(target_os = "windows")]
const ICONS: Font = Font::with_name("Segoe UI Symbol");
#[cfg(target_os = "macos")]
const ICONS: Font = Font::with_name("SF Pro");
#[cfg(target_os = "linux")]
const ICONS: Font = Font::with_name("Noto Sans Symbols");
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
const ICONS: Font = Font::DEFAULT;

const MIN_TOUCH_TARGET: f32 = 44.0;
const TAB_WIDTH: f32 = 180.0;
const ICON_SIZE: u16 = 16;
const TEXT_SIZE_NORMAL: u16 = 14;
const TEXT_SIZE_SMALL: u16 = 12;

fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("🚀 Start of FAGA Browser...");
    log::info!("📱 Initialize UI Manager with Google-like interface...");

    // Police par défaut selon l'OS
    #[cfg(target_os = "windows")]
    let default_font = Font::with_name("Segoe UI");
    #[cfg(target_os = "macos")]
    let default_font = Font::with_name("SF Pro Display");
    #[cfg(target_os = "linux")]
    let default_font = Font::with_name("Cantarell");
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_font = Font::DEFAULT;

    FagaBrowser::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            decorations: false,
            ..Default::default()
        },
        default_font,
        ..Default::default()
    })
}

/// État de chargement d'une page
#[derive(Debug, Clone)]
enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

/// Représente un onglet du navigateur
#[derive(Debug, Clone)]
struct Tab {
    id: usize,
    title: String,
    url: String,
    loading_state: LoadingState,
    content: Option<PageContent>,
    history: Vec<String>,
    history_index: usize,
}

/// Contenu d'une page web chargée avec styles CSS appliqués
#[derive(Debug, Clone)]
struct PageContent {
    document_title: String,
    styled_content: Vec<StyledText>,
    body_styles: Option<parser::renderer::ComputedStyles>,
}

impl Tab {
    fn new(id: usize) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url: "faga://newtab".to_string(),
            loading_state: LoadingState::Idle,
            content: None,
            history: vec!["faga://newtab".to_string()],
            history_index: 0,
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len().saturating_sub(1)
    }

    fn go_back(&mut self) -> Option<String> {
        if self.can_go_back() {
            self.history_index -= 1;
            Some(self.history[self.history_index].clone())
        } else {
            None
        }
    }

    fn go_forward(&mut self) -> Option<String> {
        if self.can_go_forward() {
            self.history_index += 1;
            Some(self.history[self.history_index].clone())
        } else {
            None
        }
    }

    fn navigate_to(&mut self, url: &str) {
        // Truncate forward history
        self.history.truncate(self.history_index + 1);
        self.history.push(url.to_string());
        self.history_index = self.history.len() - 1;
        self.url = url.to_string();
    }
}

struct FagaBrowser {
    tabs: Vec<Tab>,
    active_tab: usize,
    url_input: String,
    next_tab_id: usize,
    http_client: Option<HttpClient>,
    // Drag state for tab reordering
    dragging_tab: Option<DragState>,
    // DevTools state
    dev_tools_open: bool,
    dev_tools_tab: DevToolsTab,
    // Window size for viewport units (vw, vh)
    window_width: f32,
    window_height: f32,
}

/// État du drag d'un onglet
#[derive(Debug, Clone)]
struct DragState {
    tab_index: usize,
    start_x: f32,
    current_x: f32,
    offset_x: f32,
    is_dragging: bool, // true si on a commencé à vraiment drag (mouvement > seuil)
}

/// Onglets des DevTools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevToolsTab {
    Elements,
    Styles,
    Console,
    Network,
}

impl Default for DevToolsTab {
    fn default() -> Self { Self::Elements }
}

#[derive(Debug, Clone)]
enum Message {
    UrlInputChanged(String),
    Navigate,
    GoBack,
    GoForward,
    Refresh,
    NewTab,
    CloseTab(usize),
    SelectTab(usize),
    OpenShortcut(String),
    // Window controls
    MinimizeWindow,
    MaximizeWindow,
    CloseWindow,
    // Window drag (for moving the window)
    StartWindowDrag,
    // Window resize
    WindowResized(f32, f32), // width, height
    // Tab drag & drop
    TabDragStart(usize, f32), // tab_index, x position
    TabDragMove(f32),         // current x position
    TabDragEnd,
    TabDragCancel,
    // Detach tab to new window
    DetachTab(usize),
    // DevTools
    ToggleDevTools,
    SelectDevToolsTab(DevToolsTab),
    // Network events
    PageLoaded(usize, Result<PageContent, String>),
    LoadingStarted(usize),
}

impl Application for FagaBrowser {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let http_client = HttpClient::new().ok();

        (FagaBrowser {
            tabs: vec![Tab::new(0)],
            active_tab: 0,
            url_input: String::new(),
            next_tab_id: 1,
            http_client,
            dragging_tab: None,
            dev_tools_open: false,
            dev_tools_tab: DevToolsTab::default(),
            window_width: 1200.0,
            window_height: 800.0,
        }, Command::none())
    }

    fn title(&self) -> String {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            format!("{} - FAGA Browser", tab.title)
        } else {
            "FAGA Browser".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UrlInputChanged(url) => {
                self.url_input = url;
            }
            Message::Navigate => {
                let url = if self.url_input.starts_with("http://")
                    || self.url_input.starts_with("https://")
                {
                    self.url_input.clone()
                } else if self.url_input.starts_with("faga://") {
                    self.url_input.clone()
                } else if self.url_input.contains('.') {
                    format!("https://{}", self.url_input)
                } else {
                    format!("https://www.google.com/search?q={}", self.url_input)
                };

                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.navigate_to(&url);
                    tab.loading_state = LoadingState::Loading;
                    let tab_id = tab.id;
                    log::info!("🌐 Navigating to: {}", url);
                    return Self::load_page(tab_id, url, self.window_width, self.window_height);
                }
            }
            Message::GoBack => {
                let result = if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(url) = tab.go_back() {
                        tab.loading_state = LoadingState::Loading;
                        Some((tab.id, url))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((tab_id, url)) = result {
                    log::info!("⬅️ Going back to: {}", url);
                    self.url_input = url.clone();
                    return Self::load_page(tab_id, url, self.window_width, self.window_height);
                }
            }
            Message::GoForward => {
                let result = if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(url) = tab.go_forward() {
                        tab.loading_state = LoadingState::Loading;
                        Some((tab.id, url))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((tab_id, url)) = result {
                    log::info!("➡️ Going forward to: {}", url);
                    self.url_input = url.clone();
                    return Self::load_page(tab_id, url, self.window_width, self.window_height);
                }
            }
            Message::Refresh => {
                let result = if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let url = tab.url.clone();
                    tab.loading_state = LoadingState::Loading;
                    Some((tab.id, url))
                } else {
                    None
                };

                if let Some((tab_id, url)) = result {
                    log::info!("🔄 Refreshing: {}", url);
                    return Self::load_page(tab_id, url, self.window_width, self.window_height);
                }
            }
            Message::NewTab => {
                let new_tab = Tab::new(self.next_tab_id);
                self.tabs.push(new_tab);
                self.active_tab = self.tabs.len() - 1;
                self.next_tab_id += 1;
                self.url_input.clear();
                log::info!("➕ New tab created");
            }
            Message::CloseTab(id) => {
                if self.tabs.len() > 1 {
                    if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
                        self.tabs.remove(pos);
                        if self.active_tab >= self.tabs.len() {
                            self.active_tab = self.tabs.len() - 1;
                        }
                    }
                } else {
                    return window::close(window::Id::MAIN);
                }
            }
            Message::SelectTab(index) => {
                if index < self.tabs.len() {
                    self.active_tab = index;
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        self.url_input = if tab.url == "faga://newtab" {
                            String::new()
                        } else {
                            tab.url.clone()
                        };
                    }
                }
            }
            Message::OpenShortcut(url) => {
                // Résoudre les URLs relatives par rapport à l'URL de la page actuelle
                let resolved_url = if let Some(tab) = self.tabs.get(self.active_tab) {
                    resolve_url(&tab.url, &url)
                } else {
                    url.clone()
                };

                let tab_id = if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.navigate_to(&resolved_url);
                    tab.loading_state = LoadingState::Loading;
                    Some(tab.id)
                } else {
                    None
                };

                if let Some(id) = tab_id {
                    self.url_input = resolved_url.clone();
                    log::info!("🔗 Opening link: {} (resolved from {})", resolved_url, url);
                    return Self::load_page(id, resolved_url, self.window_width, self.window_height);
                }
            }
            Message::MinimizeWindow => {
                return window::minimize(window::Id::MAIN, true);
            }
            Message::MaximizeWindow => {
                return window::toggle_maximize(window::Id::MAIN);
            }
            Message::CloseWindow => {
                return window::close(window::Id::MAIN);
            }
            Message::PageLoaded(tab_id, result) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
                    match result {
                        Ok(content) => {
                            tab.title = if content.document_title.is_empty() {
                                tab.url.replace("https://", "").replace("http://", "")
                            } else {
                                content.document_title.clone()
                            };
                            tab.content = Some(content);
                            tab.loading_state = LoadingState::Loaded;
                            log::info!("✅ Page loaded successfully: {}", tab.url);
                        }
                        Err(error) => {
                            tab.loading_state = LoadingState::Error(error.clone());
                            log::error!("❌ Failed to load page: {}", error);
                        }
                    }
                }
            }
            Message::LoadingStarted(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.loading_state = LoadingState::Loading;
                }
            }
            // Window drag - déplacer la fenêtre (compatible multi-OS)
            Message::StartWindowDrag => {
                return window::drag(window::Id::MAIN);
            }
            // Window resize - mise à jour de la taille de la fenêtre
            Message::WindowResized(width, height) => {
                self.window_width = width;
                self.window_height = height;
                log::debug!("📐 Window resized: {}x{}", width, height);
            }
            // Tab drag & drop - nouveau système
            Message::TabDragStart(index, x) => {
                if index < self.tabs.len() {
                    self.dragging_tab = Some(DragState {
                        tab_index: index,
                        start_x: x,
                        current_x: x,
                        offset_x: 0.0,
                        is_dragging: false, // Pas encore vraiment en train de drag
                    });
                    log::debug!("🔄 Potential drag started for tab {} at x={}", index, x);
                }
            }
            Message::TabDragMove(x) => {
                if let Some(ref mut drag) = self.dragging_tab {
                    drag.current_x = x;
                    drag.offset_x = x - drag.start_x;

                    // Seuil de démarrage du drag (10 pixels)
                    const DRAG_THRESHOLD: f32 = 10.0;
                    if !drag.is_dragging && drag.offset_x.abs() > DRAG_THRESHOLD {
                        drag.is_dragging = true;
                        // Sélectionner aussi l'onglet qu'on drag
                        self.active_tab = drag.tab_index;
                        log::info!("🔄 Started real drag for tab {}", drag.tab_index);
                    }

                    // Seulement échanger si on est vraiment en train de drag
                    if drag.is_dragging {
                        let tab_index = drag.tab_index;
                        let offset = drag.offset_x;

                        // Si on a déplacé d'au moins 60% d'un onglet
                        if offset > TAB_WIDTH * 0.6 && tab_index < self.tabs.len() - 1 {
                            // Déplacer vers la droite
                            self.tabs.swap(tab_index, tab_index + 1);
                            drag.tab_index = tab_index + 1;
                            drag.start_x = x;
                            drag.offset_x = 0.0;

                            if self.active_tab == tab_index {
                                self.active_tab = tab_index + 1;
                            } else if self.active_tab == tab_index + 1 {
                                self.active_tab = tab_index;
                            }
                            log::info!("📋 Swapped tab {} → {}", tab_index, tab_index + 1);
                        } else if offset < -TAB_WIDTH * 0.6 && tab_index > 0 {
                            // Déplacer vers la gauche
                            self.tabs.swap(tab_index, tab_index - 1);
                            drag.tab_index = tab_index - 1;
                            drag.start_x = x;
                            drag.offset_x = 0.0;

                            if self.active_tab == tab_index {
                                self.active_tab = tab_index - 1;
                            } else if self.active_tab == tab_index - 1 {
                                self.active_tab = tab_index;
                            }
                            log::info!("📋 Swapped tab {} → {}", tab_index, tab_index - 1);
                        }
                    }
                }
            }
            Message::TabDragEnd => {
                if let Some(drag) = &self.dragging_tab {
                    // Si on n'a pas vraiment drag (juste un clic), sélectionner l'onglet
                    if !drag.is_dragging {
                        self.active_tab = drag.tab_index;
                        if let Some(tab) = self.tabs.get(self.active_tab) {
                            self.url_input = if tab.url == "faga://newtab" {
                                String::new()
                            } else {
                                tab.url.clone()
                            };
                        }
                        log::debug!("🔄 Click on tab {} (no drag)", drag.tab_index);
                    } else {
                        log::debug!("🔄 Ended tab drag at index {}", drag.tab_index);
                    }
                }
                self.dragging_tab = None;
            }
            Message::TabDragCancel => {
                self.dragging_tab = None;
                log::debug!("🔄 Cancelled tab drag");
            }
            Message::DetachTab(index) => {
                // Pour l'instant, log seulement - l'implémentation multi-fenêtre
                // nécessite une architecture plus complexe
                if index < self.tabs.len() && self.tabs.len() > 1 {
                    log::info!("🪟 Detach tab {} requested (not yet implemented)", index);
                    // TODO: Implémenter le détachement vers une nouvelle fenêtre
                    // Cela nécessite de gérer plusieurs fenêtres avec iced::multi_window
                }
            }
            // DevTools
            Message::ToggleDevTools => {
                self.dev_tools_open = !self.dev_tools_open;
                log::info!("🔧 DevTools {}", if self.dev_tools_open { "opened" } else { "closed" });
            }
            Message::SelectDevToolsTab(tab) => {
                self.dev_tools_tab = tab;
                log::debug!("🔧 DevTools tab: {:?}", tab);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Tab bar
        let tab_bar = self.view_tab_bar();

        // Navigation bar
        let nav_bar = self.view_navigation_bar();

        // Content area with optional DevTools
        let content = self.view_content();

        // Main content layout
        let page_area: Element<Message> = if self.dev_tools_open {
            // Split view: content on top, DevTools on bottom
            let dev_tools = self.view_dev_tools();
            column![
                container(content).height(Length::FillPortion(6)),
                dev_tools
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            content
        };

        let main_content = column![tab_bar, nav_bar, page_area]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    /// Subscription pour les événements clavier et souris
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status| {
            match event {
                // Window resize
                Event::Window(_, window::Event::Resized { width, height }) => {
                    Some(Message::WindowResized(width as f32, height as f32))
                }
                // Keyboard: CTRL+SHIFT+I pour ouvrir DevTools
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(c),
                    modifiers,
                    ..
                }) => {
                    if modifiers.control() && modifiers.shift() && c.as_str() == "i" {
                        return Some(Message::ToggleDevTools);
                    }
                    // F12 aussi pour ouvrir DevTools
                    None
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::F12),
                    ..
                }) => {
                    Some(Message::ToggleDevTools)
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => {
                    Some(Message::TabDragCancel)
                }
                // Mouse events for tab dragging
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::TabDragMove(position.x))
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::TabDragEnd)
                }
                _ => None,
            }
        })
    }
}

impl FagaBrowser {
    /// Load a page asynchronously (static method to avoid borrow issues)
    fn load_page(tab_id: usize, url: String, viewport_width: f32, viewport_height: f32) -> Command<Message> {
        // Handle internal URLs
        if url.starts_with("faga://") {
            return Command::perform(
                async move { Ok(PageContent {
                    document_title: "New Tab".to_string(),
                    styled_content: Vec::new(),
                    body_styles: None,
                }) },
                move |result| Message::PageLoaded(tab_id, result),
            );
        }

        // Perform HTTP request and render with CSS
        Command::perform(
            async move {
                let client = HttpClient::new()
                    .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

                let response = client.get(&url).await
                    .map_err(|e| format!("Request failed: {}", e))?;

                if !response.is_success() {
                    return Err(format!("HTTP Error: {}", response.status));
                }

                // Parse HTML
                let document = HtmlParser::parse(&response.body, &url)
                    .map_err(|e| format!("HTML parsing failed: {}", e))?;

                // Create renderer with default CSS and viewport dimensions
                let mut renderer = HtmlRenderer::new()
                    .with_viewport(viewport_width, viewport_height);

                // Add page stylesheets (inline CSS from <style> tags)
                for stylesheet in &document.stylesheets {
                    if stylesheet.starts_with("inline:") {
                        let css = &stylesheet[7..]; // Remove "inline:" prefix
                        log::info!("🎨 Adding inline CSS: {}...", &css[..css.len().min(50)]);
                        renderer.add_stylesheet(css);
                    }
                }

                // Render the document to get styled content with body styles
                let rendered = if let Some(render_tree) = renderer.render(&document) {
                    flatten_render_tree_with_body(&render_tree)
                } else {
                    parser::renderer::RenderedContent {
                        styled_content: Vec::new(),
                        body_styles: None,
                    }
                };

                // Log body styles for debugging
                if let Some(ref body) = rendered.body_styles {
                    log::info!("📐 Body styles: margin-top={}px, width={:?}, width_percent={:?}, margin_auto={}/{}",
                        body.margin_top,
                        body.width,
                        body.width_percent,
                        body.margin_left_auto,
                        body.margin_right_auto
                    );
                }

                Ok(PageContent {
                    document_title: document.title,
                    styled_content: rendered.styled_content,
                    body_styles: rendered.body_styles,
                })
            },
            move |result| Message::PageLoaded(tab_id, result),
        )
    }

    fn view_tab_bar(&self) -> Element<Message> {
        let mut tabs_row = Row::new().spacing(2).align_items(Alignment::Center);

        // Déterminer si on est en train de drag un onglet
        let dragging_index = self.dragging_tab.as_ref().map(|d| d.tab_index);
        let drag_offset = self.dragging_tab.as_ref().map(|d| d.offset_x).unwrap_or(0.0);

        for (index, tab) in self.tabs.iter().enumerate() {
            let is_active = index == self.active_tab;

            // Titre de l'onglet
            let tab_title = text(if tab.title.len() > 18 {
                format!("{}...", &tab.title[..15])
            } else {
                tab.title.clone()
            })
                .size(TEXT_SIZE_SMALL);

            // Bouton fermer
            let close_btn = button(text("×").size(14))
                .on_press(Message::CloseTab(tab.id))
                .padding(Padding::from([2, 6]))
                .style(iced::theme::Button::Custom(Box::new(TabCloseButtonStyle)));

            let tab_inner = row![
                tab_title,
                horizontal_space(),
                close_btn
            ]
                .spacing(4)
                .align_items(Alignment::Center)
                .width(Length::Fixed(TAB_WIDTH - 20.0));

            // Style selon l'état - avec effet visuel de drag
            let is_being_dragged = self.dragging_tab.as_ref()
                .map(|d| d.tab_index == index && d.is_dragging)
                .unwrap_or(false);



            // Calcul de l'offset visuel si on drag cet onglet
            let _visual_offset = if is_being_dragged { drag_offset } else { 0.0 };

            // Le bouton n'a pas de on_press car le clic est géré par mouse_area
            let tab_button = container(tab_inner)
                .padding(Padding::from([8, 10]))
                .style(iced::theme::Container::Custom(Box::new(TabButtonContainerStyle {
                    is_active,
                    is_dragging: is_being_dragged
                })));

            // Container avec possibilité de démarrer un drag
            // L'utilisateur doit maintenir le clic et bouger pour drag
            let tab_index = index;
            let start_x = (index as f32) * (TAB_WIDTH + 2.0) + TAB_WIDTH / 2.0;

            let tab_container = container(tab_button)
                .width(Length::Fixed(TAB_WIDTH));

            // Utiliser mouse_area pour détecter le press/release
            let tab_with_drag = iced::widget::mouse_area(tab_container)
                .on_press(Message::TabDragStart(tab_index, start_x))
                .on_release(Message::TabDragEnd);

            tabs_row = tabs_row.push(tab_with_drag);
        }

        // New tab button
        let new_tab_btn = button(
            container(text("+").size(18))
                .width(Length::Fixed(MIN_TOUCH_TARGET))
                .height(Length::Fixed(32.0))
                .center_x()
                .center_y()
        )
            .on_press(Message::NewTab)
            .padding(0)
            .style(iced::theme::Button::Custom(Box::new(IconButtonStyle)));

        tabs_row = tabs_row.push(new_tab_btn);

        // Zone de drag pour déplacer la fenêtre (cliquer et glisser)
        let drag_area = iced::widget::mouse_area(
            container(horizontal_space())
                .width(Length::Fill)
                .height(Length::Fixed(38.0))
        )
        .on_press(Message::StartWindowDrag);

        tabs_row = tabs_row.push(drag_area);

        // Window controls - boutons avec taille accessible et feedback visuel
        let window_controls = row![
            // Minimize button
            button(
                container(text("—").font(ICONS).size(ICON_SIZE))
                    .width(Length::Fixed(46.0))
                    .height(Length::Fixed(32.0))
                    .center_x()
                    .center_y()
            )
                .on_press(Message::MinimizeWindow)
                .padding(0)
                .style(iced::theme::Button::Custom(Box::new(WindowControlStyle))),
            // Maximize button
            button(
                container(text("☐").font(ICONS).size(ICON_SIZE))
                    .width(Length::Fixed(46.0))
                    .height(Length::Fixed(32.0))
                    .center_x()
                    .center_y()
            )
                .on_press(Message::MaximizeWindow)
                .padding(0)
                .style(iced::theme::Button::Custom(Box::new(WindowControlStyle))),
            // Close button - rouge pour signaler l'action destructive
            button(
                container(text("✕").font(ICONS).size(ICON_SIZE))
                    .width(Length::Fixed(46.0))
                    .height(Length::Fixed(32.0))
                    .center_x()
                    .center_y()
            )
                .on_press(Message::CloseWindow)
                .padding(0)
                .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle))),
        ]
        .spacing(0);

        let full_tab_bar = row![tabs_row, window_controls]
            .spacing(8)
            .align_items(Alignment::Center)
            .padding(Padding::from([6, 8]));

        container(full_tab_bar)
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(TabBarStyle)))
            .into()
    }

    fn view_navigation_bar(&self) -> Element<Message> {
        // Navigation buttons avec taille tactile minimum
        let back_btn = button(
            container(text("◀").font(ICONS).size(ICON_SIZE))
                .width(Length::Fixed(MIN_TOUCH_TARGET))
                .height(Length::Fixed(36.0))
                .center_x()
                .center_y()
        )
            .on_press(Message::GoBack)
            .padding(0)
            .style(iced::theme::Button::Custom(Box::new(NavButtonStyle)));

        let forward_btn = button(
            container(text("▶").font(ICONS).size(ICON_SIZE))
                .width(Length::Fixed(MIN_TOUCH_TARGET))
                .height(Length::Fixed(36.0))
                .center_x()
                .center_y()
        )
            .on_press(Message::GoForward)
            .padding(0)
            .style(iced::theme::Button::Custom(Box::new(NavButtonStyle)));

        let refresh_btn = button(
            container(text("⟳").font(ICONS).size(18))
                .width(Length::Fixed(MIN_TOUCH_TARGET))
                .height(Length::Fixed(36.0))
                .center_x()
                .center_y()
        )
            .on_press(Message::Refresh)
            .padding(0)
            .style(iced::theme::Button::Custom(Box::new(NavButtonStyle)));

        // URL bar - hauteur suffisante pour accessibilité
        let url_bar = text_input("Search FAGA or type a URL", &self.url_input)
            .on_input(Message::UrlInputChanged)
            .on_submit(Message::Navigate)
            .padding(Padding::from([10, 16]))
            .size(TEXT_SIZE_NORMAL)
            .width(Length::Fill);

        let url_container = container(url_bar)
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(UrlBarStyle)));

        // Menu button
        let menu_btn = button(
                container(text("⋮").font(ICONS).size(20))
                .width(Length::Fixed(MIN_TOUCH_TARGET))
                .height(Length::Fixed(36.0))
                .center_x()
                .center_y()
        )
            .on_press(Message::Refresh)
            .padding(0)
            .style(iced::theme::Button::Custom(Box::new(NavButtonStyle)));

        let nav_row = row![
            back_btn,
            forward_btn,
            refresh_btn,
            Space::with_width(8),
            url_container,
            Space::with_width(8),
            menu_btn,
        ]
        .spacing(2)
        .align_items(Alignment::Center)
        .padding(Padding::from([8, 12]));

        container(nav_row)
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(NavBarStyle)))
            .into()
    }

    fn view_content(&self) -> Element<Message> {
        let current_tab = self.tabs.get(self.active_tab);

        match current_tab {
            Some(tab) if tab.url == "faga://newtab" => {
                self.view_new_tab_page()
            }
            Some(tab) => {
                match &tab.loading_state {
                    LoadingState::Loading => {
                        // Afficher un indicateur de chargement comme Chrome
                        container(
                            column![
                                text("⟳").font(ICONS).size(48),
                                Space::with_height(16),
                                text("Loading...").size(16),
                                text(&tab.url).size(12).style(Color::from_rgb(0.5, 0.5, 0.5)),
                            ]
                            .spacing(8)
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                    }
                    LoadingState::Error(error) => {
                        // Afficher l'erreur
                        container(
                            column![
                                text("⚠").font(ICONS).size(48).style(Color::from_rgb(0.9, 0.3, 0.3)),
                                Space::with_height(16),
                                text("Failed to load page").size(20),
                                text(error).size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                                Space::with_height(16),
                                button(text("Retry").size(14))
                                    .on_press(Message::Refresh)
                                    .padding(Padding::from([10, 20]))
                                    .style(iced::theme::Button::Primary),
                            ]
                            .spacing(8)
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                    }
                    LoadingState::Loaded => {
                        // Afficher le contenu stylisé avec le CSS par défaut appliqué
                        if let Some(content) = &tab.content {
                            self.render_styled_content(content, &tab.url)
                        } else {
                            container(text("No content"))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x()
                                .center_y()
                                .into()
                        }
                    }
                    LoadingState::Idle => {
                        container(
                            column![text("Enter a URL to start browsing").size(16)]
                                .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                    }
                }
            }
            None => {
                container(text("No tab selected"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }

    /// Render styled content using the parsed CSS styles
    fn render_styled_content(&self, content: &PageContent, _url: &str) -> Element<Message> {
        // Build the content column with styled text
        let mut content_column = column![].spacing(2).width(Length::Fill);

        // Get body background color from first styled element if available
        let body_bg = content.styled_content
            .first()
            .map(|s| s.styles.background_color)
            .unwrap_or(parser::renderer::RenderColor::rgb(255, 255, 255));

        // Render each styled text segment - group by lines
        let mut current_line: Vec<Element<Message>> = Vec::new();
        let mut line_margin_top: f32 = 0.0;

        for styled in &content.styled_content {
            if styled.text == "\n" {
                // Flush current line
                if !current_line.is_empty() {
                    let line_row = Row::with_children(current_line.drain(..).collect::<Vec<Element<Message>>>() )
                        .spacing(0)
                        .align_items(Alignment::Center);

                    content_column = content_column.push(
                        container(line_row)
                            .padding(Padding::from([line_margin_top as u16, 0, 0, 0]))
                    );
                }
                line_margin_top = 0.0;
            } else {
                if styled.text.trim().is_empty() {
                    continue;
                }

                // Apply styles from CSS
                let size = (styled.styles.font_size as u16).max(10).min(72);
                let color = styled.styles.color.to_iced_color();

                // Track margins for spacing
                if line_margin_top < styled.styles.margin_top {
                    line_margin_top = styled.styles.margin_top;
                }

                // Create element - either clickable link or plain text
                let element: Element<Message> = if let Some(ref href) = styled.href {
                    // C'est un lien cliquable
                    let link_text = text(&styled.text)
                        .size(size)
                        .style(color);

                    button(link_text)
                        .on_press(Message::OpenShortcut(href.clone()))
                        .padding(0)
                        .style(iced::theme::Button::Custom(Box::new(LinkButtonStyle)))
                        .into()
                } else {
                    // Texte normal
                    text(&styled.text)
                        .size(size)
                        .style(color)
                        .into()
                };

                current_line.push(element);
            }
        }

        // Flush remaining content
        if !current_line.is_empty() {
            let line_row = Row::with_children(current_line)
                .spacing(0)
                .align_items(Alignment::Center);

            content_column = content_column.push(line_row);
        }

        // Get body styles if available
        let body = content.body_styles.as_ref();

        // Utiliser les vraies dimensions de la fenêtre pour vw et vh
        let viewport_width = self.window_width;
        let viewport_height = self.window_height;

        // Calculate width from body styles
        let content_width: Length = if let Some(body_styles) = body {
            log::info!("📐 Body width_percent: {:?}, width: {:?}, viewport: {}x{}",
                body_styles.width_percent, body_styles.width, viewport_width, viewport_height);
            if let Some(percent) = body_styles.width_percent {
                // width en vw ou % - convertir en pixels pour permettre le centrage
                let width_px = viewport_width * percent / 100.0;
                Length::Fixed(width_px)
            } else if let Some(px) = body_styles.width {
                Length::Fixed(px)
            } else {
                Length::Fill
            }
        } else {
            // Default 60vw
            Length::Fixed(viewport_width * 0.6)
        };

        // Calculate margins from body styles
        let (margin_top, margin_auto) = if let Some(body_styles) = body {
            let top = body_styles.margin_top.max(0.0) as u16;
            let auto = body_styles.margin_left_auto && body_styles.margin_right_auto;
            log::info!("📐 Body margin_top: {}px, margin_left_auto: {}, margin_right_auto: {}, auto: {}",
                top, body_styles.margin_left_auto, body_styles.margin_right_auto, auto);
            (top, auto)
        } else {
            // Default: 15vh centered
            ((viewport_height * 0.15) as u16, true)
        };

        // Create inner container with page styling
        let inner_content = container(content_column)
            .width(Length::Fill)
            .padding(Padding::from([16, 24]))
            .style(iced::theme::Container::Custom(Box::new(ContentBoxStyle)));

        // Create content container with proper width
        let content_container = container(inner_content)
            .width(content_width);

        // Apply centering if margin: auto
        let centered_content: Element<Message> = if margin_auto {
            container(content_container)
                .width(Length::Fill)
                .center_x()
                .into()
        } else {
            content_container.into()
        };

        // Create outer container with body background and margin
        let outer_container = container(centered_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([margin_top, 0, 0, 0]))
            .style(iced::theme::Container::Custom(Box::new(PageBackgroundStyle {
                color: body_bg
            })));

        scrollable(outer_container)
            .height(Length::Fill)
            .into()
    }

    /// Affiche le panneau DevTools (comme Chrome DevTools)
    fn view_dev_tools(&self) -> Element<Message> {

        // Barre d'onglets DevTools
        let tab_button = |label: &str, tab: DevToolsTab, current: DevToolsTab| {
            let is_active = tab == current;
            button(
                text(label).size(12)
            )
            .on_press(Message::SelectDevToolsTab(tab))
            .padding(Padding::from([6, 12]))
            .style(if is_active {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            })
        };

        let tabs_bar = row![
            tab_button("Elements", DevToolsTab::Elements, self.dev_tools_tab),
            tab_button("Styles", DevToolsTab::Styles, self.dev_tools_tab),
            tab_button("Console", DevToolsTab::Console, self.dev_tools_tab),
            tab_button("Network", DevToolsTab::Network, self.dev_tools_tab),
            horizontal_space(),
            button(text("×").size(14))
                .on_press(Message::ToggleDevTools)
                .padding(Padding::from([4, 8]))
                .style(iced::theme::Button::Text),
        ]
        .spacing(4)
        .padding(Padding::from([4, 8]))
        .align_items(Alignment::Center);

        // Contenu selon l'onglet sélectionné
        let content: Element<Message> = match self.dev_tools_tab {
            DevToolsTab::Elements => self.view_dev_tools_elements(),
            DevToolsTab::Styles => self.view_dev_tools_styles(),
            DevToolsTab::Console => self.view_dev_tools_console(),
            DevToolsTab::Network => self.view_dev_tools_network(),
        };

        let dev_tools_panel = column![
            container(tabs_bar)
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(DevToolsTabBarStyle))),
            content
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::FillPortion(4));

        container(dev_tools_panel)
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DevToolsPanelStyle)))
            .into()
    }

    /// Onglet Elements - affiche la structure DOM
    fn view_dev_tools_elements(&self) -> Element<Message> {
        let mut content = column![].spacing(2).padding(8);

        if let Some(tab) = self.tabs.get(self.active_tab) {
            if let Some(page_content) = &tab.content {
                content = content.push(
                    text(format!("📄 Document: {}", tab.url))
                        .size(12)
                        .style(Color::from_rgb(0.4, 0.4, 0.9))
                );
                content = content.push(Space::with_height(8));

                // Afficher les éléments stylisés avec leur structure
                for (i, styled) in page_content.styled_content.iter().take(50).enumerate() {
                    if styled.text.trim().is_empty() {
                        continue;
                    }

                    let indent = "  ".repeat(styled.depth.min(6));
                    let preview = if styled.text.len() > 60 {
                        format!("{}...", &styled.text[..57])
                    } else {
                        styled.text.clone()
                    };

                    let line = text(format!(
                        "{}[{}] \"{}\"",
                        indent,
                        i,
                        preview.replace('\n', "↵")
                    ))
                    .size(11)
                    .style(Color::from_rgb(0.3, 0.3, 0.3));

                    content = content.push(line);
                }

                if page_content.styled_content.len() > 50 {
                    content = content.push(
                        text(format!("... et {} autres éléments", page_content.styled_content.len() - 50))
                            .size(11)
                            .style(Color::from_rgb(0.5, 0.5, 0.5))
                    );
                }
            } else {
                content = content.push(text("Aucun contenu chargé").size(12));
            }
        } else {
            content = content.push(text("Aucun onglet sélectionné").size(12));
        }

        scrollable(content)
            .height(Length::Fill)
            .into()
    }

    /// Onglet Styles - affiche les styles CSS appliqués
    fn view_dev_tools_styles(&self) -> Element<Message> {
        use parser::renderer::{FontWeight};

        let mut content = column![].spacing(4).padding(8);

        content = content.push(
            text("🎨 Styles CSS appliqués")
                .size(13)
                .style(Color::from_rgb(0.2, 0.2, 0.6))
        );
        content = content.push(Space::with_height(8));

        if let Some(tab) = self.tabs.get(self.active_tab) {
            if let Some(page_content) = &tab.content {
                // Montrer les styles uniques utilisés
                let mut style_summary: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

                for styled in &page_content.styled_content {
                    let key = format!(
                        "font-size: {}px; font-weight: {}; color: rgb({},{},{})",
                        styled.styles.font_size as u32,
                        match styled.styles.font_weight {
                            FontWeight::Bold => "bold",
                            FontWeight::Normal => "normal",
                        },
                        styled.styles.color.r,
                        styled.styles.color.g,
                        styled.styles.color.b
                    );
                    *style_summary.entry(key).or_insert(0) += 1;
                }

                for (style, count) in style_summary.iter().take(20) {
                    let line = row![
                        text(format!("× {}", count))
                            .size(10)
                            .style(Color::from_rgb(0.6, 0.3, 0.1))
                            .width(Length::Fixed(40.0)),
                        text(style)
                            .size(10)
                            .style(Color::from_rgb(0.2, 0.4, 0.2))
                    ]
                    .spacing(8);

                    content = content.push(line);
                }

                // Afficher les styles de chaque élément
                content = content.push(Space::with_height(16));
                content = content.push(
                    text("📋 Détail par élément:")
                        .size(12)
                        .style(Color::from_rgb(0.3, 0.3, 0.5))
                );

                for (i, styled) in page_content.styled_content.iter().take(30).enumerate() {
                    if styled.text.trim().is_empty() {
                        continue;
                    }

                    let preview = if styled.text.len() > 30 {
                        format!("{}...", &styled.text[..27])
                    } else {
                        styled.text.replace('\n', "")
                    };

                    let style_info = format!(
                        "size: {}px, weight: {}, color: #{:02X}{:02X}{:02X}, margin-top: {}px",
                        styled.styles.font_size as u32,
                        match styled.styles.font_weight {
                            FontWeight::Bold => "bold",
                            FontWeight::Normal => "normal",
                        },
                        styled.styles.color.r,
                        styled.styles.color.g,
                        styled.styles.color.b,
                        styled.styles.margin_top as u32
                    );

                    let element_row = column![
                        text(format!("[{}] \"{}\"", i, preview))
                            .size(10)
                            .style(Color::from_rgb(0.1, 0.1, 0.4)),
                        text(format!("    {}", style_info))
                            .size(9)
                            .style(Color::from_rgb(0.4, 0.4, 0.4))
                    ]
                    .spacing(1);

                    content = content.push(element_row);
                }
            } else {
                content = content.push(text("Chargez une page pour voir les styles").size(12));
            }
        }

        scrollable(content)
            .height(Length::Fill)
            .into()
    }

    /// Onglet Console - affiche les logs
    fn view_dev_tools_console(&self) -> Element<Message> {
        let mut content = column![].spacing(4).padding(8);

        content = content.push(
            text("📝 Console (Logs)")
                .size(13)
                .style(Color::from_rgb(0.2, 0.4, 0.2))
        );
        content = content.push(Space::with_height(8));

        // Info sur l'état actuel
        content = content.push(
            text(format!("ℹ️ Onglets ouverts: {}", self.tabs.len()))
                .size(11)
                .style(Color::from_rgb(0.3, 0.3, 0.6))
        );
        content = content.push(
            text(format!("ℹ️ Onglet actif: {}", self.active_tab))
                .size(11)
                .style(Color::from_rgb(0.3, 0.3, 0.6))
        );

        if let Some(tab) = self.tabs.get(self.active_tab) {
            content = content.push(
                text(format!("ℹ️ URL: {}", tab.url))
                    .size(11)
                    .style(Color::from_rgb(0.3, 0.3, 0.6))
            );
            content = content.push(
                text(format!("ℹ️ État: {:?}", tab.loading_state))
                    .size(11)
                    .style(Color::from_rgb(0.3, 0.3, 0.6))
            );

            if let Some(page_content) = &tab.content {
                content = content.push(
                    text(format!("✅ {} éléments rendus", page_content.styled_content.len()))
                        .size(11)
                        .style(Color::from_rgb(0.2, 0.5, 0.2))
                );
            }
        }

        content = content.push(Space::with_height(16));
        content = content.push(
            text("💡 Appuyez sur F12 ou Ctrl+Shift+I pour fermer")
                .size(10)
                .style(Color::from_rgb(0.5, 0.5, 0.5))
        );

        scrollable(content)
            .height(Length::Fill)
            .into()
    }

    /// Onglet Network - affiche les requêtes réseau
    fn view_dev_tools_network(&self) -> Element<Message> {
        let mut content = column![].spacing(4).padding(8);

        content = content.push(
            text("🌐 Network (Requêtes)")
                .size(13)
                .style(Color::from_rgb(0.4, 0.2, 0.4))
        );
        content = content.push(Space::with_height(8));

        if let Some(tab) = self.tabs.get(self.active_tab) {
            let status_color = match &tab.loading_state {
                LoadingState::Loaded => Color::from_rgb(0.2, 0.6, 0.2),
                LoadingState::Loading => Color::from_rgb(0.6, 0.5, 0.1),
                LoadingState::Error(_) => Color::from_rgb(0.7, 0.2, 0.2),
                LoadingState::Idle => Color::from_rgb(0.5, 0.5, 0.5),
            };

            content = content.push(
                row![
                    text("GET").size(10).style(Color::from_rgb(0.2, 0.5, 0.2)),
                    text(&tab.url).size(10).style(Color::from_rgb(0.3, 0.3, 0.6)),
                ]
                .spacing(8)
            );

            content = content.push(
                text(format!("Status: {:?}", tab.loading_state))
                    .size(10)
                    .style(status_color)
            );

            // Historique de navigation
            if !tab.history.is_empty() {
                content = content.push(Space::with_height(12));
                content = content.push(
                    text("📜 Historique de navigation:")
                        .size(11)
                        .style(Color::from_rgb(0.4, 0.4, 0.4))
                );

                for (i, url) in tab.history.iter().enumerate() {
                    let marker = if i == tab.history_index { "▶" } else { "  " };
                    content = content.push(
                        text(format!("{} {}", marker, url))
                            .size(10)
                            .style(if i == tab.history_index {
                                Color::from_rgb(0.2, 0.4, 0.6)
                            } else {
                                Color::from_rgb(0.5, 0.5, 0.5)
                            })
                    );
                }
            }
        } else {
            content = content.push(text("Aucun onglet sélectionné").size(12));
        }

        scrollable(content)
            .height(Length::Fill)
            .into()
    }

    fn view_new_tab_page(&self) -> Element<Message> {
        // Shortcuts section
        let shortcuts = self.view_shortcuts();

        let content = column![
            // Spacer
            container(text("")).height(Length::FillPortion(2)),
            // Shortcuts
            shortcuts,
            // Spacer
            container(text("")).height(Length::FillPortion(3)),
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill);

        scrollable(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::from([40, 20]))
        )
        .height(Length::Fill)
        .into()
    }

    fn view_shortcuts(&self) -> Element<Message> {
        let shortcuts_data = vec![
            ("Project Zomboi...", "https://projectzomboid.com", "P"),
            ("Web Store", "https://chrome.google.com/webstore", "🌈"),
        ];

        let mut shortcuts_row = Row::new()
            .spacing(24)
            .align_items(Alignment::Center);

        for (name, url, icon) in shortcuts_data {
            let shortcut = self.create_shortcut(name, url, icon);
            shortcuts_row = shortcuts_row.push(shortcut);
        }

        container(shortcuts_row)
            .width(Length::Shrink)
            .center_x()
            .into()
    }

    fn create_shortcut(&self, name: &str, url: &str, icon: &str) -> Element<Message> {
        let icon_container = container(
            text(icon).size(24)
        )
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(ShortcutIconStyle)));

        let label = text(name)
            .size(12)
            .width(Length::Fixed(80.0))
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        let shortcut_content = column![icon_container, label]
            .spacing(8)
            .align_items(Alignment::Center);

        button(shortcut_content)
            .on_press(Message::OpenShortcut(url.to_string()))
            .padding(Padding::from([12, 8]))
            .style(iced::theme::Button::Text)
            .into()
    }
}

// Custom styles
struct TabBarStyle;
impl iced::widget::container::StyleSheet for TabBarStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.85, 0.89, 0.95))),
            ..Default::default()
        }
    }
}

struct NavBarStyle;
impl iced::widget::container::StyleSheet for NavBarStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.85, 0.89, 0.95))),
            ..Default::default()
        }
    }
}

// Style pour le container d'onglet (remplace les boutons)
struct TabButtonContainerStyle {
    is_active: bool,
    is_dragging: bool,
}

impl iced::widget::container::StyleSheet for TabButtonContainerStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        if self.is_dragging {
            iced::widget::container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgba(0.26, 0.52, 0.96, 0.5))),
                text_color: Some(Color::WHITE),
                border: iced::Border {
                    color: Color::from_rgb(0.26, 0.52, 0.96),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        } else if self.is_active {
            iced::widget::container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.26, 0.52, 0.96))),
                text_color: Some(Color::WHITE),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        } else {
            iced::widget::container::Appearance {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
    }
}

struct UrlBarStyle;
impl iced::widget::container::StyleSheet for UrlBarStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: iced::Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ShortcutIconStyle;
impl iced::widget::container::StyleSheet for ShortcutIconStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border: iced::Border {
                color: Color::from_rgb(0.9, 0.9, 0.9),
                width: 1.0,
                radius: 24.0.into(),
            },
            ..Default::default()
        }
    }
}

// Style pour le contenu de la page web
struct ContentBoxStyle;
impl iced::widget::container::StyleSheet for ContentBoxStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

// Style pour les liens cliquables (transparent, sans bordure)
struct LinkButtonStyle;
impl iced::widget::button::StyleSheet for LinkButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            text_color: Color::from_rgb(0.1, 0.05, 0.67), // Bleu lien
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Default::default(),
            shadow_offset: Default::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            text_color: Color::from_rgb(0.2, 0.1, 0.8), // Bleu plus clair au hover
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Default::default(),
            shadow_offset: Default::default(),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            text_color: Color::from_rgb(0.5, 0.0, 0.0), // Rouge au clic
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Default::default(),
            shadow_offset: Default::default(),
        }
    }
}

// Style pour le fond de page avec couleur CSS dynamique
struct PageBackgroundStyle {
    color: parser::renderer::RenderColor,
}
impl iced::widget::container::StyleSheet for PageBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(self.color.to_iced_color())),
            ..Default::default()
        }
    }
}

// Styles pour le panneau DevTools
struct DevToolsPanelStyle;
impl iced::widget::container::StyleSheet for DevToolsPanelStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.96, 0.96, 0.96))),
            border: iced::Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct DevToolsTabBarStyle;
impl iced::widget::container::StyleSheet for DevToolsTabBarStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.92, 0.92, 0.92))),
            border: iced::Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

// Style pour l'onglet actif
struct ActiveTabStyle;
impl iced::widget::button::StyleSheet for ActiveTabStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.26, 0.52, 0.96))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.22, 0.46, 0.88))),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.18, 0.40, 0.80))),
            ..active
        }
    }
}

// Style pour l'onglet inactif
struct InactiveTabStyle;
impl iced::widget::button::StyleSheet for InactiveTabStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgb(0.3, 0.3, 0.3),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.08))),
            text_color: Color::from_rgb(0.2, 0.2, 0.2),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.12))),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour l'onglet en cours de glissement
struct DraggingTabStyle;
impl iced::widget::button::StyleSheet for DraggingTabStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.26, 0.52, 0.96, 0.5))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::from_rgb(0.26, 0.52, 0.96),
                width: 2.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}

// Style pour les boutons de déplacement d'onglets (flèches)
struct TabMoveButtonStyle;
impl iced::widget::button::StyleSheet for TabMoveButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
            text_color: Color::from_rgb(0.2, 0.2, 0.2),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.2))),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour les boutons de déplacement désactivés
struct TabMoveButtonDisabledStyle;
impl iced::widget::button::StyleSheet for TabMoveButtonDisabledStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgba(0.6, 0.6, 0.6, 0.3),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}

// Style pour le bouton fermer d'onglet
struct TabCloseButtonStyle;
impl iced::widget::button::StyleSheet for TabCloseButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgba(0.4, 0.4, 0.4, 0.7),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.9, 0.2, 0.2, 0.2))),
            text_color: Color::from_rgb(0.8, 0.2, 0.2),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.9, 0.2, 0.2, 0.4))),
            text_color: Color::from_rgb(0.7, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour les boutons icône (+ nouvel onglet)
struct IconButtonStyle;
impl iced::widget::button::StyleSheet for IconButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.08))),
            text_color: Color::from_rgb(0.2, 0.2, 0.2),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.15))),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour les boutons de contrôle de fenêtre (minimiser, maximiser)
struct WindowControlStyle;
impl iced::widget::button::StyleSheet for WindowControlStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgb(0.3, 0.3, 0.3),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.2))),
            text_color: Color::BLACK,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour le bouton fermer (rouge au hover)
struct CloseButtonStyle;
impl iced::widget::button::StyleSheet for CloseButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgb(0.3, 0.3, 0.3),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.2, 0.2))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.75, 0.15, 0.15))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

// Style pour les boutons de navigation
struct NavButtonStyle;
impl iced::widget::button::StyleSheet for NavButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::from_rgb(0.35, 0.35, 0.35),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.08))),
            text_color: Color::from_rgb(0.15, 0.15, 0.15),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.15))),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    }
}

