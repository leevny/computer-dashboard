mod gpu;

use eframe::egui;
use std::time::{Duration, Instant};
use sysinfo::System;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

const TRAY_RGBA: &[u8] = include_bytes!("../assets/tray.rgba");

// ---- 配色：低饱和四色（OKLCH 生成，已过色觉/对比度校验），表盘画在深色玻璃上 ----
const SCRIM: egui::Color32 = egui::Color32::from_rgb(0x12, 0x16, 0x1C);
const CPU: egui::Color32 = egui::Color32::from_rgb(0x35, 0x85, 0xC9);
const MEM: egui::Color32 = egui::Color32::from_rgb(0x28, 0xA1, 0x6D);
const GPU: egui::Color32 = egui::Color32::from_rgb(0xC0, 0x85, 0x1F);
const VRAM: egui::Color32 = egui::Color32::from_rgb(0xBF, 0x6A, 0xA5);
const RED: egui::Color32 = egui::Color32::from_rgb(0xA8, 0x43, 0x43); // 红区：低调砖红
const TRACK: egui::Color32 = egui::Color32::from_rgb(0x46, 0x50, 0x5E);
const TRACK_IN: egui::Color32 = egui::Color32::from_rgb(0x33, 0x3B, 0x47);
const TICK_MAJOR: egui::Color32 = egui::Color32::from_rgb(0xD5, 0xDC, 0xE5);
const TICK_MINOR: egui::Color32 = egui::Color32::from_rgb(0x4A, 0x55, 0x63);
const TICK_IN: egui::Color32 = egui::Color32::from_rgb(0x5E, 0x6A, 0x7D);
const NUM: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x94, 0xA6);
const INK: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xED, 0xF3);

const A0: f32 = -120.0; // 表盘起始角：0° 朝上，顺时针为正
const SWEEP: f32 = 240.0;
const R_OUTER: f32 = 42.0;
const R_INNER: f32 = 22.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("sys-gauge")
            .with_always_on_top()
            .with_decorations(false)
            .with_inner_size([224.0, 112.0]),
        ..Default::default()
    };
    let mut sys = System::new().expect("sysinfo 初始化失败");
    sys.refresh_cpu_usage(); // 采样基线：读数从下一次刷新开始有效
    let gpu = gpu::Gpu::new().ok(); // ponytail: PDH 初始化失败时 GPU 指标显示 0
    let vram_total = gpu::Gpu::total_dedicated();
    let tray = TrayUi::new().ok(); // ponytail: 托盘失败仅失去快捷入口，主窗口不受影响
    eframe::run_native(
        "sys-gauge",
        options,
        Box::new(move |cc| {
            load_cjk(&cc.egui_ctx);
            if let Some(gl) = cc.gl.clone() {
                egui_bg_layer::install_glow(&cc.egui_ctx, &gl).ok();
            }
            Ok(Box::new(App {
                sys,
                gpu,
                vram_total,
                stats: Stats::default(),
                disp: [0.0; 4],
                last: Instant::now(),
                topmost: true,
                style: GlassStyle::Hybrid,
                tray,
            }))
        }),
    )
}

/// 补充 CJK 字体（默认字体不含中文，菜单/标签需要）
fn load_cjk(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in ["C:\\Windows\\Fonts\\msyh.ttc", "C:\\Windows\\Fonts\\simhei.ttf"] {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("cjk".into(), egui::FontData::from_owned(data).into());
            for fam in fonts.families.values_mut() {
                fam.push("cjk".into());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

/// 系统托盘：右下角小图标 + 右键菜单（风格切换 / 置顶 / 退出）
struct TrayUi {
    style: MenuId,
    topmost: MenuId,
    quit: MenuId,
    check_top: CheckMenuItem,
    _tray: TrayIcon,
}

impl TrayUi {
    fn new() -> Result<Self, String> {
        let style = MenuItem::new("切换玻璃风格", true, None);
        let check_top = CheckMenuItem::new("窗口置顶", true, true, None);
        let quit = MenuItem::new("退出", true, None);
        let mut menu = Menu::new();
        menu.append(&style).map_err(|e| e.to_string())?;
        menu.append(&check_top).map_err(|e| e.to_string())?;
        menu.append(&quit).map_err(|e| e.to_string())?;
        let _tray = TrayIconBuilder::new()
            .with_icon(
                tray_icon::Icon::from_rgba(TRAY_RGBA.to_vec(), 32, 32)
                    .map_err(|e| e.to_string())?,
            )
            .with_menu(Box::new(menu))
            .with_tooltip("sys-gauge")
            .with_menu_on_left_click(true)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            style: style.id().clone(),
            topmost: check_top.id().clone(),
            quit: quit.id().clone(),
            check_top,
            _tray,
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum GlassStyle {
    Frosted, // 毛玻璃：染色最淡，背景透出最多
    Liquid,  // 液态玻璃：内部光晕主导，发光玻璃面板
    Hybrid,  // 混合
}

impl GlassStyle {
    fn name(self) -> &'static str {
        match self {
            GlassStyle::Frosted => "毛玻璃",
            GlassStyle::Liquid => "液态玻璃",
            GlassStyle::Hybrid => "混合",
        }
    }
    fn next(self) -> Self {
        match self {
            GlassStyle::Frosted => GlassStyle::Liquid,
            GlassStyle::Liquid => GlassStyle::Hybrid,
            GlassStyle::Hybrid => GlassStyle::Frosted,
        }
    }
    /// (光晕 alpha, 模糊半径, 饱和度, overlay alpha)
    fn params(self) -> (f32, f32, f32, u8) {
        match self {
            GlassStyle::Frosted => (0.22, 18.0, 1.3, 200),
            GlassStyle::Liquid => (0.55, 32.0, 1.8, 150),
            GlassStyle::Hybrid => (0.40, 24.0, 1.6, 185),
        }
    }
}

fn mono(size: f32, text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).font(egui::FontId::new(size, egui::FontFamily::Monospace))
}

struct App {
    sys: System,
    gpu: Option<gpu::Gpu>,
    vram_total: u64,
    stats: Stats,
    disp: [f32; 4], // 指针平滑显示值：CPU/GPU/内存/显存
    last: Instant,
    topmost: bool,
    style: GlassStyle,
    tray: Option<TrayUi>,
}

#[derive(Default)]
struct Stats {
    cpu: f32,
    gpu_pct: f32,
    mem_pct: f32,
    mem_total_gb: f64,
    vram_pct: f32,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // 托盘菜单事件（窗口右键菜单已废弃，控制入口全部在托盘）
        let ids = self.tray.as_ref().map(|t| (t.style.clone(), t.topmost.clone(), t.quit.clone()));
        if let Some((style_id, top_id, quit_id)) = ids {
            while let Ok(ev) = MenuEvent::receiver().try_recv() {
                if ev.id == style_id {
                    self.style = self.style.next();
                } else if ev.id == top_id {
                    self.set_topmost(ui, !self.topmost);
                } else if ev.id == quit_id {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if self.last.elapsed() >= Duration::from_secs(1) {
            self.last = Instant::now();
            self.sample();
        }
        // 事件驱动：任一指针在动才重绘（30fps）；到位即休眠到下一次采样
        let targets = [self.stats.cpu, self.stats.gpu_pct, self.stats.mem_pct, self.stats.vram_pct];
        let mut moving = false;
        for (i, d) in self.disp.iter_mut().enumerate() {
            let step = (targets[i] - *d) * 0.35; // 带符号差：超过目标时向回收敛
            if step.abs() > 0.25 {
                *d += step;
                moving = true;
            } else {
                *d = targets[i];
            }
        }
        if moving {
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        } else {
            let wait = Duration::from_secs(1).saturating_sub(self.last.elapsed());
            ui.ctx().request_repaint_after(wait.max(Duration::from_millis(33)));
        }

        let (glow_a, blur_r, sat, overlay_a) = self.style.params();

        egui::Frame::NONE.fill(SCRIM).show(ui, |ui| {
            let bg_rect = ui.max_rect();
            let bp = ui.painter();
            // 光晕对齐表盘中心（窗口 224×112，两盘中心各在半区中央）
            let c1 = bg_rect.left_center() + egui::vec2(56.0, 9.0);
            let c2 = bg_rect.left_center() + egui::vec2(168.0, 9.0);
            bp.circle_filled(c1, 50.0, CPU.gamma_multiply(glow_a));
            bp.circle_filled(c2, 50.0, MEM.gamma_multiply(glow_a));
            use egui_bg_layer::{presets, BG_Panel};
            BG_Panel::new(presets::gaussian_blur(blur_r).then(presets::saturation(sat)))
                .corner_radius(12.0)
                .overlay(egui::Color32::from_rgba_unmultiplied(0x12, 0x16, 0x1C, overlay_a))
                .paint(ui);
            // egui_bg_layer 的 shader pass 不恢复 glViewport，会把后续绘制挤到屏幕外——手动还原
            let restore = egui_glow::CallbackFn::new(move |info, painter| unsafe {
                use eframe::glow::HasContext;
                let gl: &eframe::glow::Context = painter.gl();
                gl.viewport(0, 0, info.screen_size_px[0] as i32, info.screen_size_px[1] as i32);
            });
            ui.painter().add(egui::PaintCallback {
                rect: egui::Rect::EVERYTHING,
                callback: std::sync::Arc::new(restore),
            });
            // 玻璃层之上的柔光晕芯（同心圆渐变），呼应表盘
            for (cc, col) in [(c1, CPU), (c2, MEM)] {
                for k in 0..3u32 {
                    let a = (glow_a * 0.55 * (1.0 - k as f32 * 0.33)) as u8;
                    bp.circle_filled(cc, 44.0 - k as f32 * 7.0, col.gamma_multiply(a as f32 / 255.0));
                }
            }

            // 全窗拖动（无边框窗口的唯一移动方式）
            let resp = ui.interact(ui.max_rect(), egui::Id::new("drag"), egui::Sense::click_and_drag());
            if resp.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            // 两个表盘清晰地画在玻璃层之上，等分窗口
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let w = ui.available_width() / 2.0;
                let p = &ui.painter().clone();
                let (r1, _) = ui.allocate_exact_size(egui::vec2(w, 106.0), egui::Sense::hover());
                let c1 = r1.center() + egui::vec2(0.0, 4.0);
                draw_gauge(p, c1, self.disp[0], CPU, self.disp[1], GPU);
                readout(p, c1, &self.disp_text(0), CPU, &self.disp_text(1), GPU);
                let (r2, _) = ui.allocate_exact_size(egui::vec2(w, 106.0), egui::Sense::hover());
                let c2 = r2.center() + egui::vec2(0.0, 4.0);
                draw_gauge(p, c2, self.disp[2], MEM, self.disp[3], VRAM);
                readout(p, c2, &self.disp_text(2), MEM, &self.disp_text(3), VRAM);
            });
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.071, 0.086, 0.11, 1.0] // #12161C 不透明底
    }
}

impl App {
    fn target(&self, i: usize) -> f32 {
        match i {
            0 => self.stats.cpu,
            1 => self.stats.gpu_pct,
            2 => self.stats.mem_pct,
            _ => self.stats.vram_pct,
        }
    }

    fn disp_text(&self, i: usize) -> String {
        match i {
            0 => format!("{:.0}%", self.disp[0]),
            1 => format!("{:.0}%", self.disp[1]),
            2 => format!("{:.1}", self.disp[2] * self.stats.mem_total_gb as f32 / 100.0),
            _ => format!("{:.1}", self.disp[3] * self.vram_total as f32 / 100.0 / 1073741824.0),
        }
    }

    fn sample(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.stats.cpu = self.sys.global_cpu_usage();
        let mem_total = self.sys.total_memory() as f64;
        self.stats.mem_total_gb = mem_total / 1073741824.0;
        self.stats.mem_pct = (self.sys.used_memory() as f64 / mem_total * 100.0) as f32;

        let (pid_util, vram_used, _pid_vram) = match &self.gpu {
            Some(g) => g.poll(),
            None => (Default::default(), 0, Default::default()),
        };
        self.stats.gpu_pct = pid_util.values().sum::<f32>().min(100.0);
        self.stats.vram_pct = if self.vram_total > 0 {
            (vram_used as f32 / self.vram_total as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
    }

    fn set_topmost(&mut self, ui: &egui::Ui, on: bool) {
        self.topmost = on;
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            if on { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal },
        ));
        if let Some(tray) = &self.tray {
            tray.check_top.set_checked(on);
        }
    }
}

fn pt(c: egui::Pos2, deg: f32, r: f32) -> egui::Pos2 {
    let a = deg.to_radians();
    egui::pos2(c.x + r * a.sin(), c.y - r * a.cos()) // 0° 朝上、顺时针为正
}

/// 圆弧折线（约每 4° 一段）
fn arc_pts(c: egui::Pos2, r: f32, a0: f32, a1: f32) -> Vec<egui::Pos2> {
    let n = ((a1 - a0).abs() / 4.0).ceil().max(2.0) as usize;
    (0..=n).map(|i| pt(c, a0 + (a1 - a0) * i as f32 / n as f32, r)).collect()
}

fn draw_gauge(
    p: &egui::Painter,
    c: egui::Pos2,
    outer_pct: f32,
    outer_c: egui::Color32,
    inner_pct: f32,
    inner_c: egui::Color32,
) {
    let stroke = |w: f32, col: egui::Color32| egui::Stroke::new(w, col);
    let ang = |v: f32| A0 + SWEEP * v / 100.0;
    let outer_pct = outer_pct.clamp(0.0, 100.0);
    let inner_pct = inner_pct.clamp(0.0, 100.0);

    // 底弧 + 红区（外圈 85–100%）
    p.line(arc_pts(c, R_OUTER, A0, ang(100.0)), stroke(6.5, TRACK));
    p.line(arc_pts(c, R_OUTER + 6.5, ang(85.0), ang(100.0)), stroke(3.0, RED));
    p.line(arc_pts(c, R_INNER, A0, ang(100.0)), stroke(3.5, TRACK_IN));

    // 刻度：外每 5%（25% 带数字），内只放主刻度
    for v in (0..=100).step_by(5) {
        let a = ang(v as f32);
        let major = v % 25 == 0;
        p.line_segment(
            [pt(c, a, if major { 31.0 } else { 34.5 }), pt(c, a, 38.0)],
            stroke(if major { 2.0 } else { 1.2 }, if major { TICK_MAJOR } else { TICK_MINOR }),
        );
        if major {
            p.text(pt(c, a, 27.0), egui::Align2::CENTER_CENTER, v.to_string(),
                egui::FontId::monospace(8.0), NUM);
            p.line_segment([pt(c, a, 18.0), pt(c, a, 23.0)], stroke(1.8, TICK_IN));
        }
    }

    // 进度弧：外粗内细
    if outer_pct > 1.0 {
        p.line(arc_pts(c, R_OUTER, A0, ang(outer_pct)), stroke(6.5, outer_c));
    }
    if inner_pct > 1.0 {
        p.line(arc_pts(c, R_INNER, A0, ang(inner_pct)), stroke(3.5, inner_c));
    }
    // 双针：外长粗、内短细
    p.line_segment([pt(c, ang(outer_pct), 9.0), pt(c, ang(outer_pct), 34.0)], stroke(2.8, outer_c));
    p.line_segment([pt(c, ang(inner_pct), 8.0), pt(c, ang(inner_pct), 26.0)], stroke(1.8, inner_c));
    p.circle(c, 4.2, egui::Color32::from_rgb(0x10, 0x14, 0x1A), stroke(1.2, TICK_MINOR));
}

/// 表盘底部开口区的双列数字读数
fn readout(p: &egui::Painter, c: egui::Pos2, outer: &str, outer_c: egui::Color32, inner: &str, inner_c: egui::Color32) {
    for (dx, val, lc) in [(-22.0, outer, outer_c), (22.0, inner, inner_c)] {
        p.text(c + egui::vec2(dx, 28.0), egui::Align2::CENTER_CENTER, val,
            egui::FontId::monospace(13.0), lc);
    }
}
