#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 双击 exe 不弹黑窗

mod gpu;

use eframe::egui;
use std::time::{Duration, Instant};
use sysinfo::System;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

const TRAY_RGBA: &[u8] = include_bytes!("../assets/tray.rgba");

const A0: f32 = -120.0; // 表盘起始角：0° 朝上，顺时针为正
const SWEEP: f32 = 240.0;
const R_OUTER: f32 = 42.0;
const R_INNER: f32 = 22.0;

// ---- 配色：两套主题均为低饱和四色（OKLCH 生成，已过色觉/对比度校验） ----
struct Palette {
    cpu: egui::Color32,
    gpu: egui::Color32,
    mem: egui::Color32,
    vram: egui::Color32,
    red: egui::Color32,
    track: egui::Color32,
    track_in: egui::Color32,
    tick_major: egui::Color32,
    tick_minor: egui::Color32,
    tick_in: egui::Color32,
    num: egui::Color32,
    ink: egui::Color32,
    scrim: egui::Color32,
    glass_tint: egui::Color32,
}

const DARK: Palette = Palette {
    cpu: egui::Color32::from_rgb(0x35, 0x85, 0xC9),
    gpu: egui::Color32::from_rgb(0xC0, 0x85, 0x1F),
    mem: egui::Color32::from_rgb(0x28, 0xA1, 0x6D),
    vram: egui::Color32::from_rgb(0xBF, 0x6A, 0xA5),
    red: egui::Color32::from_rgb(0xA8, 0x43, 0x43),
    track: egui::Color32::from_rgb(0x46, 0x50, 0x5E),
    track_in: egui::Color32::from_rgb(0x33, 0x3B, 0x47),
    tick_major: egui::Color32::from_rgb(0xD5, 0xDC, 0xE5),
    tick_minor: egui::Color32::from_rgb(0x4A, 0x55, 0x63),
    tick_in: egui::Color32::from_rgb(0x5E, 0x6A, 0x7D),
    num: egui::Color32::from_rgb(0x8A, 0x94, 0xA6),
    ink: egui::Color32::from_rgb(0xE6, 0xED, 0xF3),
    scrim: egui::Color32::from_rgb(0x12, 0x16, 0x1C),
    glass_tint: egui::Color32::from_rgb(0x12, 0x16, 0x1C),
};

const LIGHT: Palette = Palette {
    cpu: egui::Color32::from_rgb(0x23, 0x76, 0xB9),
    gpu: egui::Color32::from_rgb(0x9D, 0x64, 0x00),
    mem: egui::Color32::from_rgb(0x00, 0x88, 0x56),
    vram: egui::Color32::from_rgb(0xA2, 0x50, 0x89),
    red: egui::Color32::from_rgb(0xB3, 0x40, 0x3C),
    track: egui::Color32::from_rgb(0xC3, 0xCC, 0xD8),
    track_in: egui::Color32::from_rgb(0xA8, 0xB3, 0xC1),
    tick_major: egui::Color32::from_rgb(0x3D, 0x46, 0x54),
    tick_minor: egui::Color32::from_rgb(0xB5, 0xBF, 0xCA),
    tick_in: egui::Color32::from_rgb(0x92, 0x9D, 0xAB),
    num: egui::Color32::from_rgb(0x5A, 0x64, 0x72),
    ink: egui::Color32::from_rgb(0x1F, 0x26, 0x30),
    scrim: egui::Color32::from_rgb(0xE9, 0xED, 0xF2),
    glass_tint: egui::Color32::from_rgb(0xE9, 0xED, 0xF2),
};

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Dark,
    Light,
}

impl Theme {
    fn name(self) -> &'static str {
        match self {
            Theme::Dark => "深色主题",
            Theme::Light => "浅色主题",
        }
    }
    fn next(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
    fn palette(self) -> &'static Palette {
        match self {
            Theme::Dark => &DARK,
            Theme::Light => &LIGHT,
        }
    }
    fn ordinal(self) -> usize {
        match self {
            Theme::Dark => 0,
            Theme::Light => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum GlassStyle {
    Liquid,  // 液态玻璃：光晕最浓
    Hybrid,  // 混合：中等光晕
}

impl GlassStyle {
    const ALL: [GlassStyle; 2] = [GlassStyle::Liquid, GlassStyle::Hybrid];

    fn name(self) -> &'static str {
        match self {
            GlassStyle::Liquid => "液态玻璃",
            GlassStyle::Hybrid => "混合玻璃",
        }
    }
    fn ordinal(self) -> usize {
        GlassStyle::ALL.iter().position(|s| *s == self).unwrap()
    }
    /// (光晕 alpha, 模糊半径, 饱和度, overlay alpha)
    fn params(self) -> (f32, f32, f32, u8) {
        match self {
            GlassStyle::Liquid => (0.55, 32.0, 1.8, 150),
            GlassStyle::Hybrid => (0.40, 24.0, 1.6, 185),
        }
    }
}

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
    eframe::run_native(
        "sys-gauge",
        options,
        Box::new(move |cc| {
            load_cjk(&cc.egui_ctx);
            let tray = TrayUi::new(&cc.egui_ctx).ok(); // ponytail: 托盘失败仅失去快捷入口，主窗口不受影响
            if let Some(gl) = cc.gl.clone() {
                egui_bg_layer::install_glow(&cc.egui_ctx, &gl).ok();
            }
            Ok(Box::new(App {
                sys,
                gpu,
                vram_total,
                stats: Stats::default(),
                last: Instant::now(),
                topmost: true,
                style: GlassStyle::Hybrid,
                theme: Theme::Light,
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

/// 调整整窗 uniform alpha（255 = 不透明）
fn set_uniform_alpha(frame: &eframe::Frame, alpha: u8) {
    #[allow(deprecated)]
    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
    #[allow(deprecated)]
    let Ok(RawWindowHandle::Win32(w)) = HasRawWindowHandle::raw_window_handle(frame) else { return };
    let hwnd = HWND(w.hwnd.get() as *mut core::ffi::c_void);
    unsafe {
        SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), alpha, LWA_ALPHA);
    }
}

/// 系统托盘：右下角小图标 + 右键菜单（玻璃风格单选 / 主题单选 / 置顶 / 退出）
struct TrayUi {
    style_items: Vec<CheckMenuItem>,
    theme_items: Vec<CheckMenuItem>,
    topmost: MenuId,
    check_top: CheckMenuItem,
    quit: MenuId,
    events: std::sync::Arc<std::sync::Mutex<Vec<MenuEvent>>>,
    _tray: TrayIcon,
}

impl TrayUi {
    fn new(ctx: &egui::Context) -> Result<Self, String> {
        let style_items: Vec<CheckMenuItem> = GlassStyle::ALL
            .iter()
            .map(|s| CheckMenuItem::new(s.name(), true, *s == GlassStyle::Hybrid, None))
            .collect();
        let theme_items: Vec<CheckMenuItem> = [Theme::Dark, Theme::Light]
            .iter()
            .map(|t| CheckMenuItem::new(t.name(), true, *t == Theme::Dark, None))
            .collect();
        let check_top = CheckMenuItem::new("窗口置顶", true, true, None);
        let quit = MenuItem::new("退出", true, None);
        let style_sub = Submenu::new("玻璃风格", true);
        let theme_sub = Submenu::new("主题", true);
        for it in &style_items {
            style_sub.append(it).map_err(|e| e.to_string())?;
        }
        for it in &theme_items {
            theme_sub.append(it).map_err(|e| e.to_string())?;
        }
        let mut menu = Menu::new();
        menu.append(&style_sub).map_err(|e| e.to_string())?;
        menu.append(&theme_sub).map_err(|e| e.to_string())?;
        menu.append(&quit).map_err(|e| e.to_string())?;
        menu.append(&check_top).map_err(|e| e.to_string())?;
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let queue = events.clone();
        let repaint_ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |ev| {
            queue.lock().unwrap().push(ev);
            repaint_ctx.request_repaint(); // 菜单点击立即唤醒休眠中的窗口
        }));
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
            style_items,
            theme_items,
            topmost: check_top.id().clone(),
            check_top,
            quit: quit.id().clone(),
            events,
            _tray,
        })
    }

    /// 同步单选组勾选状态到当前风格/主题
    fn sync_checks(&self, style: GlassStyle, theme: Theme) {
        for (i, it) in self.style_items.iter().enumerate() {
            it.set_checked(i == style.ordinal());
        }
        for (i, it) in self.theme_items.iter().enumerate() {
            it.set_checked(i == theme.ordinal());
        }
    }
}

struct App {
    sys: System,
    gpu: Option<gpu::Gpu>,
    vram_total: u64,
    stats: Stats,
    last: Instant,
    topmost: bool,
    style: GlassStyle,
    theme: Theme,
    tray: Option<TrayUi>,
}

#[derive(Default)]
struct Stats {
    cpu: f32,
    gpu_pct: f32,
    mem_pct: f32,
    vram_pct: f32,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let _ = frame;
        // 托盘菜单事件（控制入口全部在托盘）
        let events: Vec<MenuEvent> = self.tray.as_ref()
            .map(|t| std::mem::take(&mut *t.events.lock().unwrap()))
            .unwrap_or_default();
        for ev in events {
            if let Some(tray) = &self.tray {
                for (i, item) in tray.style_items.iter().enumerate() {
                    if ev.id == *item.id() {
                        self.style = GlassStyle::ALL[i];
                    }
                }
                for (i, item) in tray.theme_items.iter().enumerate() {
                    if ev.id == *item.id() {
                        self.theme = [Theme::Dark, Theme::Light][i];
                    }
                }
                if ev.id == tray.topmost {
                    self.set_topmost(ui, !self.topmost);
                } else if ev.id == tray.quit {
                    // ponytail: eframe 退出流程销毁 TrayIcon 时与托盘消息窗口死锁（实测卡死），直接结束进程
                    std::process::exit(0);
                }
            }
        }
        if let Some(tray) = &self.tray {
            tray.sync_checks(self.style, self.theme);
        }
        if self.last.elapsed() >= Duration::from_secs(1) {
            self.last = Instant::now();
            self.sample();
        }
        // 指针动画：egui 内置 1s 插值（新值从当前动画值续接，动画期自动重绘，结束自动休眠）
        let targets = [self.stats.cpu, self.stats.gpu_pct, self.stats.mem_pct, self.stats.vram_pct];
        let anim = |key: &str, v: f32| ui.ctx().animate_value_with_time(egui::Id::new(key), v, 1.0);
        let disp = [
            anim("anim-cpu", targets[0]),
            anim("anim-gpu", targets[1]),
            anim("anim-mem", targets[2]),
            anim("anim-vram", targets[3]),
        ];
        // 采样计时驱动下一帧（动画期间由 animate_value_with_time 自己安排重绘）
        let wait = Duration::from_secs(1).saturating_sub(self.last.elapsed());
        ui.ctx().request_repaint_after(wait.max(Duration::from_millis(33)));

        let pal = self.theme.palette();
        let (glow_a, blur_r, sat, overlay_a) = self.style.params();

        egui::Frame::NONE.fill(pal.scrim).show(ui, |ui| {
            let bg_rect = ui.max_rect();
            let bp = ui.painter();
            // 光晕对齐表盘中心（窗口 224×112，两盘中心各在半区中央）
            let c1 = bg_rect.left_center() + egui::vec2(56.0, 9.0);
            let c2 = bg_rect.left_center() + egui::vec2(168.0, 9.0);
            bp.circle_filled(c1, 50.0, pal.cpu.gamma_multiply(glow_a));
            bp.circle_filled(c2, 50.0, pal.mem.gamma_multiply(glow_a));
            if blur_r > 0.0 {
                use egui_bg_layer::{presets, BG_Panel};
                BG_Panel::new(presets::gaussian_blur(blur_r).then(presets::saturation(sat)))
                    .corner_radius(12.0)
                    .overlay(egui::Color32::from_rgba_unmultiplied(
                        pal.glass_tint.r(),
                        pal.glass_tint.g(),
                        pal.glass_tint.b(),
                        overlay_a,
                    ))
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
                for (cc, col) in [(c1, pal.cpu), (c2, pal.mem)] {
                    for k in 0..3u32 {
                        let a = (glow_a * 0.55 * (1.0 - k as f32 * 0.33)) as u8;
                        bp.circle_filled(cc, 44.0 - k as f32 * 7.0, col.gamma_multiply(a as f32 / 255.0));
                    }
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
                draw_gauge(p, c1, disp[0], pal.cpu, disp[1], pal.gpu, pal);
                readout(p, c1, ("CPU", disp[0], pal.cpu), ("GPU", disp[1], pal.gpu));
                let (r2, _) = ui.allocate_exact_size(egui::vec2(w, 106.0), egui::Sense::hover());
                let c2 = r2.center() + egui::vec2(0.0, 4.0);
                draw_gauge(p, c2, disp[2], pal.mem, disp[3], pal.vram, pal);
                readout(p, c2, ("内存", disp[2], pal.mem), ("显存", disp[3], pal.vram));
            });
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.071, 0.086, 0.11, 1.0] // 不透明底
    }
}

impl App {
    fn sample(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.stats.cpu = self.sys.global_cpu_usage();
        let mem_total = self.sys.total_memory() as f64;
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
    pal: &Palette,
) {
    let stroke = |w: f32, col: egui::Color32| egui::Stroke::new(w, col);
    let ang = |v: f32| A0 + SWEEP * v / 100.0;
    let outer_pct = outer_pct.clamp(0.0, 100.0);
    let inner_pct = inner_pct.clamp(0.0, 100.0);

    // 底弧 + 红区（外圈 85–100%）
    p.line(arc_pts(c, R_OUTER, A0, ang(100.0)), stroke(6.5, pal.track));
    p.line(arc_pts(c, R_OUTER + 6.5, ang(85.0), ang(100.0)), stroke(3.0, pal.red));
    p.line(arc_pts(c, R_INNER, A0, ang(100.0)), stroke(3.5, pal.track_in));

    // 刻度：外每 5%（25% 带数字），内只放主刻度
    for v in (0..=100).step_by(5) {
        let a = ang(v as f32);
        let major = v % 25 == 0;
        p.line_segment(
            [pt(c, a, if major { 31.0 } else { 34.5 }), pt(c, a, 38.0)],
            stroke(if major { 2.0 } else { 1.2 }, if major { pal.tick_major } else { pal.tick_minor }),
        );
        if major {
            p.text(pt(c, a, 27.0), egui::Align2::CENTER_CENTER, v.to_string(),
                egui::FontId::monospace(8.0), pal.num);
            p.line_segment([pt(c, a, 18.0), pt(c, a, 23.0)], stroke(1.8, pal.tick_in));
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
    p.circle(c, 4.2, egui::Color32::from_rgb(0x10, 0x14, 0x1A), stroke(1.2, pal.tick_minor));
}

/// 表盘底部开口区：双列「指标名 + 百分比」，名字与指针同色
fn readout(
    p: &egui::Painter,
    c: egui::Pos2,
    outer: (&str, f32, egui::Color32),
    inner: (&str, f32, egui::Color32),
) {
    for (dx, (name, val, col)) in [(-22.0, outer), (22.0, inner)] {
        p.text(c + egui::vec2(dx, 26.0), egui::Align2::CENTER_CENTER, format!("{val:.0}%"),
            egui::FontId::monospace(13.0), col);
        p.text(c + egui::vec2(dx, 40.0), egui::Align2::CENTER_CENTER, name,
            egui::FontId::proportional(8.0), col);
    }
}
