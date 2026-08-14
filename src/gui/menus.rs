use std::borrow::Cow;

use egui::Ui;
#[cfg(egui_desktop)]
use egui_desktop::{
    KeyboardShortcut, MenuItem, SubMenuItem, TitleBar, TitleBarOptions, render_resize_handles,
    titlebar::HamburgerStyle,
};

pub(super) struct Menus {
    #[cfg(egui_desktop)]
    renderer: DesktopMenusRenderer,
    #[cfg(not(egui_desktop))]
    renderer: MenusRenderer,
}

pub type MenuCallback = Box<dyn Fn() + Send + Sync>;

pub struct MenuCallbacks {
    pub rebuild: MenuCallback,
    pub export: MenuCallback,
    pub capture: MenuCallback,
    pub set_in_point: MenuCallback,
    pub set_out_point: MenuCallback,
    pub reset_in_out_point: MenuCallback,
    pub clear_font_cache: MenuCallback,
    pub subitem_selector: MenuCallback,
    pub profiler_first: MenuCallback,
    pub profiler_second: MenuCallback,
    pub rich_text_editor: MenuCallback,
    pub font_list: MenuCallback,
    pub color: MenuCallback,
    pub copy_time_point: MenuCallback,
}

impl Menus {
    pub fn new(callbacks: MenuCallbacks) -> Self {
        let file = vec![
            MenuElement::button(t!("Rebuild"), callbacks.rebuild, Some("Ctrl+L")),
            MenuElement::Separator,
            MenuElement::button(t!("Export"), callbacks.export, Some("Ctrl+S")),
            MenuElement::button(t!("Capture"), callbacks.capture, Some("Ctrl+Alt+S")),
            MenuElement::Separator,
            MenuElement::button(t!("Set In Point"), callbacks.set_in_point, Some("[")),
            MenuElement::button(t!("Set Out Point"), callbacks.set_out_point, Some("]")),
            MenuElement::button(t!("Reset In/Out Point"), callbacks.reset_in_out_point, None),
            MenuElement::Separator,
            MenuElement::button(t!("Clear Font Cache"), callbacks.clear_font_cache, None),
        ];
        let tools = vec![
            MenuElement::button(
                t!("Subitem selector"),
                callbacks.subitem_selector,
                Some("Ctrl+I"),
            ),
            MenuElement::Separator,
            MenuElement::button(t!("Profiler"), callbacks.profiler_first, None),
            MenuElement::Separator,
            MenuElement::button(t!("Draw"), callbacks.profiler_second, Some("Ctrl+D")),
            MenuElement::button(
                t!("Rich text editor"),
                callbacks.rich_text_editor,
                Some("Ctrl+R"),
            ),
            MenuElement::button(t!("Font list"), callbacks.font_list, Some("Ctrl+F")),
            MenuElement::button(t!("Color"), callbacks.color, Some("Ctrl+O")),
            MenuElement::Separator,
            MenuElement::button(t!("Copy time point"), callbacks.copy_time_point, Some("T")),
        ];

        let menus_info = vec![
            MenuInfo {
                atoms: t!("File").into_owned(),
                elements: file,
            },
            MenuInfo {
                atoms: t!("Tools").into_owned(),
                elements: tools,
            },
        ];

        Self {
            #[cfg(egui_desktop)]
            renderer: DesktopMenusRenderer::new(menus_info),
            #[cfg(not(egui_desktop))]
            renderer: MenusRenderer { menus_info },
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        self.renderer.render(ui);
    }
}

struct MenuInfo {
    pub atoms: String,
    pub elements: Vec<MenuElement>,
}

enum MenuElement {
    Button(ButtonInfo),
    Separator,
}

impl MenuElement {
    fn button(
        atoms: Cow<'_, str>,
        callback: MenuCallback,
        app_shortcut: Option<&'static str>,
    ) -> Self {
        Self::Button(ButtonInfo {
            atoms: atoms.into_owned(),
            callback,
            app_shortcut,
        })
    }
}

struct ButtonInfo {
    pub atoms: String,
    pub callback: MenuCallback,
    #[allow(unused)]
    pub app_shortcut: Option<&'static str>,
}

// ===== Menus Renderer =====

#[cfg(egui_desktop)]
struct DesktopMenusRenderer {
    title_bar: TitleBar,
    previous_window_size: egui::Vec2,
}

#[cfg(egui_desktop)]
impl DesktopMenusRenderer {
    pub(super) fn render(&mut self, ui: &mut egui::Ui) {
        let current_size = ui.input(|i| i.content_rect().size());
        if current_size != self.previous_window_size {
            self.title_bar.close_all_menus();
            self.previous_window_size = current_size;
        }

        render_resize_handles(ui);

        self.title_bar.show(ui);
    }

    fn new(menus_info: Vec<MenuInfo>) -> Self {
        use super::assets::APP_ICON;

        let mut title_bar = TitleBar::new(
            TitleBarOptions::new()
                .with_hamburger_style(HamburgerStyle::Static)
                .with_title("JAnim Graphics"),
        )
        .with_app_icon(APP_ICON, "app_icon.png");

        for menu_info in menus_info {
            let menu = Self::build_menu_item(menu_info);
            title_bar = title_bar.add_menu_with_submenu(menu);
        }

        Self {
            title_bar,
            previous_window_size: egui::Vec2::default(),
        }
    }

    fn build_menu_item(menu_info: MenuInfo) -> MenuItem {
        let mut menu_item = MenuItem::new(&menu_info.atoms);
        let mut pending_sub_item: Option<SubMenuItem> = None;

        for element in menu_info.elements {
            match element {
                MenuElement::Button(button) => {
                    if let Some(sub_item) = pending_sub_item.take() {
                        menu_item = menu_item.add_subitem(sub_item);
                    }

                    let callback = button.callback;
                    let mut item = SubMenuItem::new(&button.atoms).with_callback(callback);
                    if let Some(shortcut) = button.app_shortcut {
                        item = item.with_shortcut(KeyboardShortcut::parse(shortcut))
                    }
                    pending_sub_item = Some(item);
                }
                MenuElement::Separator => {
                    if let Some(sub_item) = pending_sub_item.take() {
                        pending_sub_item = Some(sub_item.with_separator());
                    }
                }
            }
        }

        if let Some(sub_item) = pending_sub_item {
            menu_item = menu_item.add_subitem(sub_item);
        }

        menu_item
    }
}

#[cfg(not(egui_desktop))]
struct MenusRenderer {
    pub menus_info: Vec<MenuInfo>,
}

#[cfg(not(egui_desktop))]
impl MenusRenderer {
    pub(super) fn render(&mut self, ctx: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                for menu_info in &self.menus_info {
                    self.menu_group(ui, menu_info);
                }
            });
        });
    }

    fn menu_group(&self, ui: &mut egui::Ui, menu_info: &MenuInfo) {
        ui.menu_button(&menu_info.atoms, |ui| {
            #[allow(clippy::needless_ifs)]
            for element in &menu_info.elements {
                match element {
                    MenuElement::Button(button) => {
                        if ui.button(&button.atoms).clicked() {
                            (button.callback)();
                        }
                    }
                    MenuElement::Separator => {
                        ui.separator();
                    }
                }
            }
        });
    }
}
