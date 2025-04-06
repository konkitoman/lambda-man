use eframe::egui;

use lambda_man_engine::Expr;

pub struct App {
    expr: Expr,
    input: String,
    debug: bool,

    scene_rect: egui::Rect,
    frame: usize,
}

impl Default for App {
    fn default() -> Self {
        let expr = Expr::parse("a:b:b").unwrap();
        Self {
            input: expr.format(0),
            expr,
            debug: false,
            scene_rect: egui::Rect::ZERO,
            frame: 0,
        }
    }
}

pub fn gen_rects(expr: &Expr, cursor: egui::Pos2) -> Vec<(bool, egui::Rect)> {
    let mut out = Vec::default();

    match expr {
        Expr::Group(exprs) => {
            let mut cursor = cursor;
            let start = cursor;
            let mut max_y = cursor.y;
            let mut min = start.x + 20.;

            let mut to_connect = Vec::with_capacity(exprs.len());

            for expr in exprs.iter() {
                let results = gen_rects(expr, cursor);
                let (_, last) = results.last().unwrap();
                to_connect.push((out.len() + results.len()) - 1);
                min = min.max(last.min.x + 20.);
                let mut min_size_x = 0.0f32;
                let mut max = last.max;
                for (_, rect) in results.iter() {
                    max_y = max_y.max(rect.max.y + 40.);
                    max = max.max(rect.max);
                    min_size_x = min_size_x.max(rect.max.x);
                }
                cursor.x += (min_size_x - cursor.x) + 80.;
                out.extend(results);
            }

            for id in to_connect {
                if out[id].1.size().x == 20. {
                    out[id].1.max.y = max_y - 40.;
                } else {
                    out.push((
                        false,
                        egui::Rect::from_min_size(
                            out[id].1.min,
                            egui::vec2(20., (max_y - out[id].1.min.y) - 40.),
                        ),
                    ));
                }
            }

            // this is has size width 0 so will not be visibile, this is here to add some space for every group.
            out.push((
                false,
                egui::Rect::from_min_size(egui::pos2(start.x, max_y - 20.), egui::vec2(0., 40.)),
            ));

            out.push((
                true,
                egui::Rect::from_min_size(
                    egui::pos2(start.x, max_y - 40.),
                    egui::vec2(min - start.x, 20.),
                ),
            ));
        }
        Expr::Def(expr) => {
            let results = gen_rects(expr, cursor + egui::vec2(0., 40.));
            let (extend, _) = results.last().unwrap();
            let mut max = egui::Vec2::ZERO;

            for (_, rect) in results.iter() {
                max = max.max(rect.max.to_vec2());
            }

            let rect = if *extend {
                egui::Rect::from_min_size(
                    cursor - egui::vec2(20., 20.),
                    egui::vec2((max.x - cursor.x) + 40., 20.),
                )
            } else {
                egui::Rect::from_min_size(
                    cursor - egui::vec2(20., 20.),
                    egui::vec2((max.x - cursor.x) + 20., 20.),
                )
            };

            out.push((false, rect));

            out.extend(results);

            out.last_mut().unwrap().0 = false;
        }
        Expr::Relative(id) => {
            let offset = 40. * ((*id + 1) as f32);
            out.push((
                true,
                egui::Rect::from_min_size(cursor - egui::vec2(0., offset), egui::vec2(20., offset)),
            ));
        }
        Expr::Label(label) => {
            // I don't know how to show this.

            out.push((
                false,
                egui::Rect::from_min_size(cursor, egui::vec2(20., 20.)),
            ));

            out.push((
                false,
                egui::Rect::from_min_size(cursor + egui::vec2(-20., 20.), egui::vec2(60., 20.)),
            ));

            out.push((
                true,
                egui::Rect::from_min_size(cursor + egui::vec2(0., 40.), egui::vec2(20., 20.)),
            ));
        }
    }

    out
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.debug, "Debug");
                if ui.text_edit_singleline(&mut self.input).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    if let Some(expr) = Expr::parse(&self.input) {
                        self.expr = expr;
                        self.input = self.expr.format(0);
                        self.scene_rect = egui::Rect::ZERO;
                        self.frame = 0;
                    }
                }
            });

            egui::Scene::new().show(ui, &mut self.scene_rect, |ui| {
                let rects = gen_rects(&self.expr, egui::pos2(60., 50.));
                let len = rects.len();

                let mut min = egui::pos2(0., 0.);
                let mut max = egui::pos2(0., 0.);

                let backgound_sid = ui.painter().add(egui::Shape::Noop);

                for (i, (_, rect)) in rects.into_iter().enumerate() {
                    let color = if self.debug {
                        egui::Color32::from(egui::epaint::ecolor::Hsva::new(
                            i as f32 / (len as f32),
                            1.,
                            1.,
                            0.25,
                        ))
                    } else {
                        egui::Color32::WHITE
                    };

                    if self.frame != 0 {
                        ui.painter().rect_filled(rect, 0., color);
                    }

                    min = min.min(rect.min);
                    max = max.max(rect.max);
                }

                let rect = egui::Rect::from_min_max(min, max + egui::vec2(40., 40.));

                ui.allocate_exact_size(rect.size(), egui::Sense::empty());

                if self.frame != 0 {
                    ui.painter().set(
                        backgound_sid,
                        egui::epaint::RectShape::new(
                            rect,
                            0.,
                            egui::Color32::BLACK,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                            egui::StrokeKind::Middle,
                        ),
                    );
                }
            });

            self.frame += 1;
        });
    }
}

fn main() {
    eframe::run_native(
        "Lambda Man Tromp",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::default()))),
    )
    .unwrap();
}
