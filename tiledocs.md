heres an example on how to use the egui dock crate
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashSet;

use eframe::NativeOptions;
use egui::{
    color_picker::{color_edit_button_srgba, Alpha},
    vec2, CentralPanel, ComboBox, Frame, Rounding, Slider, TopBottomPanel, Ui, ViewportBuilder,
    WidgetText,
};

use egui_dock::{
    AllowedSplits, DockArea, DockState, NodeIndex, OverlayType, Style, SurfaceIndex,
    TabInteractionStyle, TabViewer,
};

/// Adds a widget with a label next to it, can be given an extra parameter in order to show a hover text
macro_rules! labeled_widget {
    ($ui:expr, $x:expr, $l:expr) => {
        $ui.horizontal(|ui| {
            ui.add($x);
            ui.label($l);
        });
    };
    ($ui:expr, $x:expr, $l:expr, $d:expr) => {
        $ui.horizontal(|ui| {
            ui.add($x).on_hover_text($d);
            ui.label($l).on_hover_text($d);
        });
    };
}

// Creates a slider which has a unit attached to it
// When given an extra parameter it will be used as a multiplier (e.g 100.0 when working with percentages)
macro_rules! unit_slider {
    ($val:expr, $range:expr) => {
        egui::Slider::new($val, $range)
    };
    ($val:expr, $range:expr, $unit:expr) => {
        egui::Slider::new($val, $range).custom_formatter(|value, decimal_range| {
            egui::emath::format_with_decimals_in_range(value, decimal_range) + $unit
        })
    };
    ($val:expr, $range:expr, $unit:expr, $mul:expr) => {
        egui::Slider::new($val, $range)
            .custom_formatter(|value, decimal_range| {
                egui::emath::format_with_decimals_in_range(value * $mul, decimal_range) + $unit
            })
            .custom_parser(|string| string.parse::<f64>().ok().map(|valid| valid / $mul))
    };
}

fn main() -> eframe::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(vec2(1024.0, 1024.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyContext {
    pub title: String,
    pub age: u32,
    pub style: Option<Style>,
    open_tabs: HashSet<String>,

    show_close_buttons: bool,
    show_add_buttons: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    allowed_splits: AllowedSplits,
    show_leaf_close_all: bool,
    show_leaf_collapse: bool,
    show_secondary_button_hint: bool,
    secondary_button_on_modifier: bool,
    secondary_button_context_menu: bool,
}

struct MyApp {
    context: MyContext,
    tree: DockState<String>,
}

impl TabViewer for MyContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Simple Demo" => self.simple_demo(ui),
            "Style Editor" => self.style_editor(ui),
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        match tab.as_str() {
            "Simple Demo" => self.simple_demo_menu(ui),
            _ => {
                ui.label(tab.to_string());
                ui.label("This is a context menu");
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        ["Inspector", "Style Editor"].contains(&tab.as_str())
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}

impl MyContext {
    fn simple_demo_menu(&mut self, ui: &mut Ui) {
        ui.label("Egui widget example");
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }

    fn simple_demo(&mut self, ui: &mut Ui) {
        ui.heading("My egui Application");

        ui.horizontal(|ui| {
            ui.label("Your name: ");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.add(Slider::new(&mut self.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            self.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", &self.title, &self.age));
    }

    fn style_editor(&mut self, ui: &mut Ui) {
        ui.heading("Style Editor");

        ui.collapsing("DockArea Options", |ui| {
            ui.checkbox(&mut self.show_close_buttons, "Show close buttons");
            ui.checkbox(&mut self.show_add_buttons, "Show add buttons");
            ui.checkbox(&mut self.draggable_tabs, "Draggable tabs");
            ui.checkbox(&mut self.show_tab_name_on_hover, "Show tab name on hover");
            ui.checkbox(
                &mut self.show_leaf_close_all,
                "Show close all button on tab bars",
            );
            ui.checkbox(
                &mut self.show_leaf_collapse,
                "Show collaspse button on tab bars",
            );
            ui.checkbox(
                &mut self.secondary_button_on_modifier,
                "Enable secondary buttons when modifiers (Shift by default) are pressed",
            );
            ui.checkbox(
                &mut self.secondary_button_context_menu,
                "Enable secondary buttons in right-click context menus",
            );
            ui.checkbox(
                &mut self.show_secondary_button_hint,
                "Show tooltip hints for secondary buttons",
            );
            ComboBox::new("cbox:allowed_splits", "Split direction(s)")
                .selected_text(format!("{:?}", self.allowed_splits))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.allowed_splits, AllowedSplits::All, "All");
                    ui.selectable_value(
                        &mut self.allowed_splits,
                        AllowedSplits::LeftRightOnly,
                        "LeftRightOnly",
                    );
                    ui.selectable_value(
                        &mut self.allowed_splits,
                        AllowedSplits::TopBottomOnly,
                        "TopBottomOnly",
                    );
                    ui.selectable_value(&mut self.allowed_splits, AllowedSplits::None, "None");
                });
        });

        let style = self.style.as_mut().unwrap();

        ui.collapsing("Border", |ui| {
            egui::Grid::new("border").show(ui, |ui| {
                ui.label("Width:");
                ui.add(Slider::new(
                    &mut style.main_surface_border_stroke.width,
                    1.0..=50.0,
                ));
                ui.end_row();

                ui.label("Color:");
                color_edit_button_srgba(
                    ui,
                    &mut style.main_surface_border_stroke.color,
                    Alpha::OnlyBlend,
                );
                ui.end_row();

                ui.label("Rounding:");
                rounding_ui(ui, &mut style.main_surface_border_rounding);
                ui.end_row();
            });
        });

        ui.collapsing("Separator", |ui| {
            egui::Grid::new("separator").show(ui, |ui| {
                ui.label("Width:");
                ui.add(Slider::new(&mut style.separator.width, 1.0..=50.0));
                ui.end_row();

                ui.label("Extra Interact Width:");
                ui.add(Slider::new(
                    &mut style.separator.extra_interact_width,
                    0.0..=50.0,
                ));
                ui.end_row();

                ui.label("Offset limit:");
                ui.add(Slider::new(&mut style.separator.extra, 1.0..=300.0));
                ui.end_row();

                ui.label("Idle color:");
                color_edit_button_srgba(ui, &mut style.separator.color_idle, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Hovered color:");
                color_edit_button_srgba(ui, &mut style.separator.color_hovered, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Dragged color:");
                color_edit_button_srgba(ui, &mut style.separator.color_dragged, Alpha::OnlyBlend);
                ui.end_row();
            });
        });

        ui.collapsing("Tabs", |ui| {
            ui.separator();

            ui.checkbox(&mut style.tab_bar.fill_tab_bar, "Expand tabs");
            ui.checkbox(
                &mut style.tab_bar.show_scroll_bar_on_overflow,
                "Show scroll bar on tab overflow",
            );
            ui.checkbox(
                &mut style.tab.hline_below_active_tab_name,
                "Show a line below the active tab name",
            );
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut style.tab_bar.height, 20.0..=50.0));
                ui.label("Tab bar height");
            });

            ComboBox::new("add_button_align", "Add button align")
                .selected_text(format!("{:?}", style.buttons.add_tab_align))
                .show_ui(ui, |ui| {
                    for align in [egui_dock::TabAddAlign::Left, egui_dock::TabAddAlign::Right] {
                        ui.selectable_value(
                            &mut style.buttons.add_tab_align,
                            align,
                            format!("{:?}", align),
                        );
                    }
                });

            ui.separator();

            fn tab_style_editor_ui(ui: &mut Ui, tab_style: &mut TabInteractionStyle) {
                ui.separator();

                ui.label("Rounding");
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.nw, 0.0..=15.0),
                    "North-West"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.ne, 0.0..=15.0),
                    "North-East"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.sw, 0.0..=15.0),
                    "South-West"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.se, 0.0..=15.0),
                    "South-East"
                );

                ui.separator();

                egui::Grid::new("tabs_colors").show(ui, |ui| {
                    ui.label("Title text color:");
                    color_edit_button_srgba(ui, &mut tab_style.text_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Outline color:")
                        .on_hover_text("The outline around the active tab name.");
                    color_edit_button_srgba(ui, &mut tab_style.outline_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Background color:");
                    color_edit_button_srgba(ui, &mut tab_style.bg_fill, Alpha::OnlyBlend);
                    ui.end_row();
                });
            }

            ui.collapsing("Active", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.active);
            });

            ui.collapsing("Inactive", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.inactive);
            });

            ui.collapsing("Focused", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.focused);
            });

            ui.collapsing("Hovered", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.hovered);
            });

            ui.separator();

            egui::Grid::new("tabs_colors").show(ui, |ui| {
                ui.label("Close button color unfocused:");
                color_edit_button_srgba(ui, &mut style.buttons.close_tab_color, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Close button color focused:");
                color_edit_button_srgba(
                    ui,
                    &mut style.buttons.close_tab_active_color,
                    Alpha::OnlyBlend,
                );
                ui.end_row();

                ui.label("Close button background color:");
                color_edit_button_srgba(ui, &mut style.buttons.close_tab_bg_fill, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Bar background color:");
                color_edit_button_srgba(ui, &mut style.tab_bar.bg_fill, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Horizontal line color:").on_hover_text(
                    "The line separating the tab name area from the tab content area",
                );
                color_edit_button_srgba(ui, &mut style.tab_bar.hline_color, Alpha::OnlyBlend);
                ui.end_row();
            });
        });

        ui.collapsing("Tab body", |ui| {
            ui.separator();

            ui.label("Rounding");
            rounding_ui(ui, &mut style.tab.tab_body.rounding);

            ui.label("Stroke width:");
            ui.add(Slider::new(
                &mut style.tab.tab_body.stroke.width,
                0.0..=10.0,
            ));
            ui.end_row();

            egui::Grid::new("tab_body_colors").show(ui, |ui| {
                ui.label("Stroke color:");
                color_edit_button_srgba(ui, &mut style.tab.tab_body.stroke.color, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Background color:");
                color_edit_button_srgba(ui, &mut style.tab.tab_body.bg_fill, Alpha::OnlyBlend);
                ui.end_row();
            });
        });
        ui.collapsing("Overlay", |ui| {
            let selected_text = match style.overlay.overlay_type {
                OverlayType::HighlightedAreas => "Highlighted Areas",
                OverlayType::Widgets => "Widgets",
            };
            ui.label("Overlay Style:");
            ComboBox::new("overlay styles", "")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut style.overlay.overlay_type,
                        OverlayType::HighlightedAreas,
                        "Highlighted Areas",
                    );
                    ui.selectable_value(
                        &mut style.overlay.overlay_type,
                        OverlayType::Widgets,
                        "Widgets",
                    );
                });
            ui.collapsing("Feel", |ui|{
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.center_drop_coverage, 0.0..=1.0, "%", 100.0),
                    "Center drop coverage",
                    "how big the area where dropping a tab into the center of another should be."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.fade_hold_time, 0.0..=4.0, "s"),
                    "Fade hold time",
                    "How long faded windows should hold their fade before unfading, in seconds."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.max_preference_time, 0.0..=4.0, "s"),
                    "Max preference time",
                    "How long the overlay may prefer to stick to a surface despite hovering over another, in seconds."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.window_drop_coverage, 0.0..=1.0, "%", 100.0),
                    "Window drop coverage",
                    "How big the area for undocking a window should be. [is overshadowed by center drop coverage]"
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.interact_expansion, 1.0..=100.0, "ps"),
                    "Interact expansion",
                    "How much extra interaction area should be allocated for buttons on the overlay"
                );
            });

            ui.collapsing("Visuals", |ui|{
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.max_button_size, 10.0..=500.0, "ps"),
                    "Max button size",
                    "The max length of a side on a overlay button in egui points"
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.button_spacing, 0.0..=50.0, "ps"),
                    "Button spacing",
                    "Spacing between buttons on the overlay, in egui units."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.surface_fade_opacity, 0.0..=1.0, "%", 100.0),
                    "Window fade opacity",
                    "how visible windows are when dragging a tab behind them."
                );
                labeled_widget!(
                    ui,
                    egui::Slider::new(&mut style.overlay.selection_stroke_width, 0.0..=50.0),
                    "Selection stroke width",
                    "width of a selection which uses a outline stroke instead of filled rect."
                );
                egui::Grid::new("overlay style preferences").show(ui, |ui| {
                    ui.label("Button color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button border color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_border_stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Selection color:");
                    color_edit_button_srgba(ui, &mut style.overlay.selection_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button stroke color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_border_stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button stroke width:");
                    ui.add(Slider::new(&mut style.overlay.button_border_stroke.width, 0.0..=50.0));
                    ui.end_row();
                });
            });

            ui.collapsing("Hover highlight", |ui|{
                egui::Grid::new("leaf highlighting prefs").show(ui, |ui|{
                    ui.label("Fill color:");
                    color_edit_button_srgba(ui, &mut style.overlay.hovered_leaf_highlight.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Stroke color:");
                    color_edit_button_srgba(ui, &mut style.overlay.hovered_leaf_highlight.stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Stroke width:");
                    ui.add(Slider::new(&mut style.overlay.hovered_leaf_highlight.stroke.width, 0.0..=50.0));
                    ui.end_row();

                    ui.label("Expansion:");
                    ui.add(Slider::new(&mut style.overlay.hovered_leaf_highlight.expansion, -50.0..=50.0));
                    ui.end_row();
                });
                ui.label("Rounding:");
                rounding_ui(ui, &mut style.overlay.hovered_leaf_highlight.rounding);
            })
        });
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let mut dock_state =
            DockState::new(vec!["Simple Demo".to_owned(), "Style Editor".to_owned()]);
        "Undock".clone_into(&mut dock_state.translations.tab_context_menu.eject_button);
        let [a, b] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["Inspector".to_owned()],
        );
        let [_, _] = dock_state.main_surface_mut().split_below(
            a,
            0.7,
            vec!["File Browser".to_owned(), "Asset Manager".to_owned()],
        );
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(b, 0.5, vec!["Hierarchy".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }
        let context = MyContext {
            title: "Hello".to_string(),
            age: 24,
            style: None,
            open_tabs,

            show_leaf_close_all: true,
            show_leaf_collapse: true,
            show_secondary_button_hint: true,
            secondary_button_on_modifier: true,
            secondary_button_context_menu: true,
            show_close_buttons: true,
            show_add_buttons: false,
            draggable_tabs: true,
            show_tab_name_on_hover: false,
            allowed_splits: AllowedSplits::default(),
        };

        Self {
            context,
            tree: dock_state,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in &["File Browser", "Asset Manager"] {
                        if ui
                            .selectable_label(self.context.open_tabs.contains(*tab), *tab)
                            .clicked()
                        {
                            if let Some(index) = self.tree.find_tab(&tab.to_string()) {
                                self.tree.remove_tab(index);
                                self.context.open_tabs.remove(*tab);
                            } else {
                                self.tree[SurfaceIndex::main()]
                                    .push_to_focused_leaf(tab.to_string());
                            }

                            ui.close_menu();
                        }
                    }
                });
            })
        });
        CentralPanel::default()
            // When displaying a DockArea in another UI, it looks better
            // to set inner margins to 0.
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let style = self
                    .context
                    .style
                    .get_or_insert(Style::from_egui(ui.style()))
                    .clone();

                DockArea::new(&mut self.tree)
                    .style(style)
                    .show_close_buttons(self.context.show_close_buttons)
                    .show_add_buttons(self.context.show_add_buttons)
                    .draggable_tabs(self.context.draggable_tabs)
                    .show_tab_name_on_hover(self.context.show_tab_name_on_hover)
                    .allowed_splits(self.context.allowed_splits)
                    .show_leaf_close_all_buttons(self.context.show_leaf_close_all)
                    .show_leaf_collapse_buttons(self.context.show_leaf_collapse)
                    .show_secondary_button_hint(self.context.show_secondary_button_hint)
                    .secondary_button_on_modifier(self.context.secondary_button_on_modifier)
                    .secondary_button_context_menu(self.context.secondary_button_context_menu)
                    .show_inside(ui, &mut self.context);
            });
    }
}

fn rounding_ui(ui: &mut Ui, rounding: &mut Rounding) {
    labeled_widget!(ui, Slider::new(&mut rounding.nw, 0.0..=15.0), "North-West");
    labeled_widget!(ui, Slider::new(&mut rounding.ne, 0.0..=15.0), "North-East");
    labeled_widget!(ui, Slider::new(&mut rounding.sw, 0.0..=15.0), "South-West");
    labeled_widget!(ui, Slider::new(&mut rounding.se, 0.0..=15.0), "South-East");
}
```

docs

Expand description

[§](#egui_dock-docking-support-for-egui)`egui_dock`: docking support for `egui`
-------------------------------------------------------------------------------

Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`. It lets you open and close tabs, freely move them around, resize them, and undock them into new egui windows which can also have other tabs docked in them.

### [§](#basic-usage)Basic usage

The library is centered around the [`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html "struct egui_dock::dock_state::DockState"). It contains a series of [`Surface`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/surface/enum.Surface.html "enum egui_dock::dock_state::surface::Surface")s which all have their own [`Tree`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/tree/struct.Tree.html "struct egui_dock::dock_state::tree::Tree"). Each [`Tree`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/tree/struct.Tree.html "struct egui_dock::dock_state::tree::Tree") stores a hierarchy of [`Node`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/tree/node/enum.Node.html "enum egui_dock::dock_state::tree::node::Node")s which contain the splits and tabs.

[`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html "struct egui_dock::dock_state::DockState") is generic (`DockState<Tab>`) so you can use any data to represent a tab. You show the tabs using [`DockArea`](https://docs.rs/egui_dock/latest/egui_dock/widgets/dock_area/struct.DockArea.html "struct egui_dock::widgets::dock_area::DockArea") and specify how they are shown by implementing [`TabViewer`](https://docs.rs/egui_dock/latest/egui_dock/widgets/tab_viewer/trait.TabViewer.html "trait egui_dock::widgets::tab_viewer::TabViewer").

```rust
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use egui::{Ui, WidgetText};

// First, let's pick a type that we'll use to attach some data to each tab.
// It can be any type.
type Tab = String;

// To define the contents and properties of individual tabs, we implement the `TabViewer`
// trait. Only three things are mandatory: the `Tab` associated type, and the `ui` and
// `title` methods. There are more methods in `TabViewer` which you can also override.
struct MyTabViewer;

impl TabViewer for MyTabViewer {
    // This associated type is used to attach some data to each tab.
    type Tab = Tab;

    // Returns the current `tab`'s title.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }
}

// Here is a simple example of how you can manage a `DockState` of your application.
struct MyTabs {
    dock_state: DockState<Tab>
}

impl MyTabs {
    pub fn new() -> Self {
        // Create a `DockState` with an initial tab "tab1" in the main `Surface`'s root node.
        let tabs = ["tab1", "tab2", "tab3"].map(str::to_string).into_iter().collect();
        let dock_state = DockState::new(tabs);
        Self { dock_state }
    }

    fn ui(&mut self, ui: &mut Ui) {
        // Here we just display the `DockState` using a `DockArea`.
        // This is where egui handles rendering and all the integrations.
        //
        // We can specify a custom `Style` for the `DockArea`, or just inherit
        // all of it from egui.
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut MyTabViewer);
    }
}

```


### [§](#look-and-feel-customization)Look and feel customization

`egui_dock` exposes the [`Style`](https://docs.rs/egui_dock/latest/egui_dock/style/struct.Style.html "struct egui_dock::style::Style") struct that lets you change how tabs and the [`DockArea`](https://docs.rs/egui_dock/latest/egui_dock/widgets/dock_area/struct.DockArea.html "struct egui_dock::widgets::dock_area::DockArea") should look and feel. [`Style`](https://docs.rs/egui_dock/latest/egui_dock/style/struct.Style.html "struct egui_dock::style::Style") is divided into several, more specialized structs that handle individual elements of the UI.

Your [`Style`](https://docs.rs/egui_dock/latest/egui_dock/style/struct.Style.html "struct egui_dock::style::Style") can inherit all its properties from an [`egui::Style`](https://docs.rs/egui/0.30.0/x86_64-unknown-linux-gnu/egui/style/struct.Style.html "struct egui::style::Style") through the [`Style::from_egui`](https://docs.rs/egui_dock/latest/egui_dock/style/struct.Style.html#method.from_egui "associated function egui_dock::style::Style::from_egui") function.

Example:

```rust
// Inherit the look and feel from egui.
let mut style = Style::from_egui(ui.style());

// Modify a few fields.
style.overlay.overlay_type = OverlayType::HighlightedAreas;
style.buttons.add_tab_align = TabAddAlign::Left;

// Use the style with the `DockArea`.
DockArea::new(&mut dock_state)
    .style(style)
    .show_inside(ui, &mut MyTabViewer);
```


### [§](#surfaces)Surfaces

A [`Surface`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/surface/enum.Surface.html "enum egui_dock::dock_state::surface::Surface") is an abstraction for any tab hierarchy. There are two kinds of non-empty surfaces: `Main` and `Window`.

There can only be one `Main` surface. It’s the one surface that is rendered inside the [`Ui`](https://docs.rs/egui/0.30.0/x86_64-unknown-linux-gnu/egui/ui/struct.Ui.html "struct egui::ui::Ui") you’ve passed to [`DockArea::show_inside`](https://docs.rs/egui_dock/latest/egui_dock/widgets/dock_area/struct.DockArea.html#method.show_inside "method egui_dock::widgets::dock_area::DockArea::show_inside"), or inside the [`egui::CentralPanel`](https://docs.rs/egui/0.30.0/x86_64-unknown-linux-gnu/egui/containers/panel/struct.CentralPanel.html "struct egui::containers::panel::CentralPanel") created by [`DockArea::show`](https://docs.rs/egui_dock/latest/egui_dock/widgets/dock_area/struct.DockArea.html#method.show "method egui_dock::widgets::dock_area::DockArea::show").

On the other hand, there can be multiple `Window` surfaces. Those represent surfaces that were created by undocking tabs from the `Main` surface, and each of them is rendered inside a [`egui::Window`](https://docs.rs/egui/0.30.0/x86_64-unknown-linux-gnu/egui/containers/window/struct.Window.html "struct egui::containers::window::Window") - hence their name.

While most of surface management will be done by the user of your application, you can also do it programatically using the [`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html "struct egui_dock::dock_state::DockState") API.

Example:

```rust
// Create a new window `Surface` with one tab inside it.
let mut surface_index = dock_state.add_window(vec!["Window Tab".to_string()]);

// Access the window state by its surface index and then move and resize it.
let window_state = dock_state.get_window_state_mut(surface_index).unwrap();
window_state.set_position(Pos2::ZERO);
window_state.set_size(Vec2::splat(100.0));
```


For more details, see: [`DockState`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/struct.DockState.html "struct egui_dock::dock_state::DockState").

### [§](#trees)Trees

In each [`Surface`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/surface/enum.Surface.html "enum egui_dock::dock_state::surface::Surface") there is a [`Tree`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/tree/struct.Tree.html "struct egui_dock::dock_state::tree::Tree") which actually stores the tabs. As the name suggests, tabs and splits are represented with a binary tree.

The [`Tree`](https://docs.rs/egui_dock/latest/egui_dock/dock_state/tree/struct.Tree.html "struct egui_dock::dock_state::tree::Tree") API allows you to programatically manipulate the dock layout.

Example:

```rust
// Create a `DockState` with an initial tab "tab1" in the main `Surface`'s root node.
let mut dock_state = DockState::new(vec!["tab1".to_string()]);

// Currently, the `DockState` only has one `Surface`: the main one.
// Let's get mutable access to add more nodes in it.
let surface = dock_state.main_surface_mut();

// Insert "tab2" to the left of "tab1", where the width of "tab2"
// is 20% of root node's width.
let [_old_node, new_node] =
    surface.split_left(NodeIndex::root(), 0.20, vec!["tab2".to_string()]);

// Insert "tab3" below "tab2" with both tabs having equal size.
surface.split_below(new_node, 0.5, vec!["tab3".to_string()]);

// The layout will look similar to this:
// +--------+--------------------------------+
// |        |                                |
// |  tab2  |                                |
// |        |                                |
// +--------+              tab1              |
// |        |                                |
// |  tab3  |                                |
// |        |                                |
// +--------+--------------------------------+
```


