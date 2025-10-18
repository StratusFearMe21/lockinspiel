use std::time::Duration;

use egui::{Align2, FontId, RichText, Vec2};
use egui_taffy::{
    TuiBuilderLogic,
    taffy::{self, Style, prelude::*},
    tui,
};
use lockinspiel_common::{
    client::LockinspielClient,
    db::{Database, JiffTimestamp},
};

enum TimerState {
    Paused(jiff::SignedDuration),
    Going(jiff::Timestamp),
}

pub struct LockinspielApp {
    timer_state: TimerState,
    client: LockinspielClient,
    runtime: tokio::runtime::Runtime,
    timers: Vec<jiff::SignedDuration>,
    timer_on: usize,
    group: Option<i64>,
    db: Database,
}

impl LockinspielApp {
    fn default() -> Self {
        let db = Database::default().unwrap();
        let mut client = LockinspielClient::default();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let now = runtime.block_on(client.now());
        let timers = vec![
            jiff::SignedDuration::from_mins(90),
            jiff::SignedDuration::from_mins(10),
        ];
        Self {
            timer_state: db
                .get()
                .unwrap()
                .get_active_timer(now)
                .unwrap()
                .map(|span| TimerState::Going(span.end_time.0))
                .unwrap_or_else(|| TimerState::Paused(timers[0])),
            client,
            runtime,
            timers,
            timer_on: 0,
            group: None,
            db,
        }
    }
}

impl LockinspielApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        cc.egui_ctx.options_mut(|options| {
            options.max_passes = std::num::NonZeroUsize::new(3).unwrap();
        });
        cc.egui_ctx.style_mut(|style| {
            style.wrap_mode = Some(egui::TextWrapMode::Extend);
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        Self::default()
    }
}

impl eframe::App for LockinspielApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                if ui.button("Sign In").clicked() {}
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            powered_by_egui_and_eframe(ui);
            egui::warn_if_debug_build(ui);
        });

        let now = self.runtime.block_on(self.client.now());

        let time_remaining = match self.timer_state {
            TimerState::Going(timer_end) => {
                let time_remaining = now.duration_until(timer_end);
                ctx.request_repaint_after(Duration::from_millis(
                    time_remaining.as_millis() as u64 % 1000,
                ));
                time_remaining
            }
            TimerState::Paused(duration) => duration,
        };

        egui::Window::new("Lockinspiel")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .auto_sized()
            .show(ctx, |ui| {
                let default_style = || taffy::Style {
                    // padding: length(8.),
                    gap: length(8.),
                    ..Default::default()
                };

                tui(ui, ui.id().with("central_panel"))
                    .style(Style {
                        flex_direction: taffy::FlexDirection::Column,
                        justify_content: Some(taffy::AlignContent::Center),
                        padding: length(8.),
                        ..default_style()
                    })
                    .show(|tui| {
                        let time_remaining_secs = time_remaining.as_secs();
                        tui.style(taffy::Style {
                            align_self: Some(taffy::AlignItems::Center),
                            ..default_style()
                        })
                        .egui_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .label(
                            RichText::new(format!(
                                "{}:{:02}",
                                time_remaining_secs / 60,
                                time_remaining_secs % 60,
                            ))
                            .font(FontId::proportional(72.0)),
                        );
                        tui.style(Style {
                            flex_direction: taffy::FlexDirection::Row,
                            align_items: Some(taffy::AlignItems::Stretch),
                            // size: taffy::Size {
                            //     width: percent(1.),
                            //     height: auto(),
                            // },
                            ..default_style()
                        })
                        .add(|tui| match self.timer_state {
                            TimerState::Going(_) => {
                                if tui
                                    .style(Style {
                                        flex_grow: 1.,
                                        ..default_style()
                                    })
                                    .ui_add(egui::Button::new("Pause"))
                                    .clicked()
                                {
                                    let db = self.db.get().unwrap();
                                    db.stop_timer(now).unwrap();
                                    self.timer_state = TimerState::Paused(time_remaining);
                                }
                                tui.enabled_ui(false)
                                    .style(Style {
                                        flex_grow: 1.,
                                        ..default_style()
                                    })
                                    .ui_add(egui::Button::new(">>"));
                            }
                            TimerState::Paused(timer_len) => {
                                if tui
                                    .style(Style {
                                        flex_grow: 1.,
                                        ..default_style()
                                    })
                                    .ui_add(egui::Button::new("Start"))
                                    .clicked()
                                {
                                    let db = self.db.get().unwrap();
                                    let end_time = now + timer_len;
                                    db.add_to_timesheet(lockinspiel_common::db::TimesheetRow {
                                        group: *self.group.get_or_insert_with(|| {
                                            db.next_timesheet_group().unwrap()
                                        }),
                                        start_time: JiffTimestamp(now),
                                        end_time: JiffTimestamp(end_time),
                                        activity: self.timer_on as i32 + 1,
                                    })
                                    .unwrap();
                                    self.timer_state = TimerState::Going(end_time);
                                }
                                if tui
                                    .enabled_ui(true)
                                    .style(Style {
                                        flex_grow: 1.,
                                        ..default_style()
                                    })
                                    .ui_add(egui::Button::new(">>"))
                                    .clicked()
                                {
                                    self.timer_on += 1;
                                    if self.timer_on >= self.timers.len() {
                                        self.timer_on = 0;
                                    }
                                    self.timer_state =
                                        TimerState::Paused(self.timers[self.timer_on]);
                                }
                            }
                        })
                    })
            });

        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
