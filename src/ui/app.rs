use crate::detector::DEBOUNCE_DURATION_SECS;
use crate::detector::{gpu::GpuType, HardwareDetector};
use crate::format_mb_size;
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// 进度更新消息
type ProgressMessage = (f32, String);

/// 硬件检测工具主应用程序
pub struct HardwareMasterApp {
    /// 硬件检测器
    detector: Arc<Mutex<HardwareDetector>>,
    /// 是否正在检测
    is_detecting: bool,
    /// 检测进度 (0.0 - 1.0)
    detection_progress: f32,
    /// 检测消息
    detection_message: String,
    /// 是否已检测
    has_detected: bool,
    /// 检测完成接收器
    detection_rx: Option<mpsc::Receiver<()>>,
    /// 进度更新接收器
    progress_rx: Option<mpsc::Receiver<ProgressMessage>>,
    /// 上次刷新时间（用于防抖）
    last_refresh_time: Option<Instant>,
    /// 当前主题
    theme: crate::ui::theme::AppTheme,
}

impl HardwareMasterApp {
    /// 创建新的应用程序实例
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置中文字体（使用跨平台字体加载）
        crate::ui::setup_chinese_fonts(&cc.egui_ctx);

        // 应用默认主题
        let theme = crate::ui::theme::AppTheme::default();
        theme.apply(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();
        let (progress_tx, progress_rx) = mpsc::channel();

        let detector = Arc::new(Mutex::new(HardwareDetector::new()));

        // 启动时自动检测
        Self::start_auto_detection_thread(Arc::clone(&detector), tx, progress_tx);

        Self {
            detector,
            is_detecting: true,
            detection_progress: 0.0,
            detection_message: "正在初始化硬件检测...".to_string(),
            has_detected: false,
            detection_rx: Some(rx),
            progress_rx: Some(progress_rx),
            last_refresh_time: None,
            theme,
        }
    }

    /// 启动自动检测线程
    fn start_auto_detection_thread(
        detector: Arc<Mutex<HardwareDetector>>,
        tx: mpsc::Sender<()>,
        progress_tx: mpsc::Sender<ProgressMessage>,
    ) {
        std::thread::spawn(move || {
            // 设置进度回调
            {
                let mut det = detector.lock().expect("硬件检测器互斥锁被污染");
                let tx_clone = progress_tx.clone();
                det.set_progress_callback(Box::new(move |progress, message| {
                    let _ = tx_clone.send((progress, message.to_string()));
                }));
            }

            // 执行检测
            let mut det = detector.lock().expect("硬件检测器互斥锁被污染");
            if let Err(e) = det.detect_all() {
                log::warn!("{}", e);
            }

            // 通知检测完成
            let _ = tx.send(());
        });
    }

    /// 开始刷新检测
    fn start_refresh(&mut self) {
        // 防抖：如果距离上次刷新不足指定时间，则忽略
        if let Some(last_time) = self.last_refresh_time {
            if last_time.elapsed() < Duration::from_secs(DEBOUNCE_DURATION_SECS) {
                return;
            }
        }

        self.last_refresh_time = Some(Instant::now());
        self.is_detecting = true;
        self.detection_progress = 0.0;
        self.detection_message = "正在重新检测硬件...".to_string();
        self.has_detected = false;

        let (tx, rx) = mpsc::channel();
        let (progress_tx, progress_rx) = mpsc::channel();
        self.detection_rx = Some(rx);
        self.progress_rx = Some(progress_rx);
        Self::start_auto_detection_thread(Arc::clone(&self.detector), tx, progress_tx);
    }

    /// 获取硬件信息文本格式
    fn get_hardware_info_text(&self) -> String {
        let detector = self.detector.lock().expect("硬件检测器互斥锁被污染");
        let mut text = String::from("以下硬件信息来源于硬大师，仅供参考：\n");

        // 操作系统
        text.push_str(&format!("操作系统: {}\n", detector.system_info.os_name));

        // 处理器
        text.push_str(&format!("处理器: {}\n", detector.cpu_info.name));

        // 显卡
        for gpu in detector.gpu_info.gpus.iter() {
            if gpu.gpu_type != GpuType::DiscreteGpu && gpu.gpu_type != GpuType::IntegratedGpu {
                continue;
            }
            text.push_str(&format!(
                "{}: {} ({}, {})\n",
                gpu.gpu_type.to_string(),
                gpu.description,
                format_mb_size!(gpu.vram_size),
                gpu.manufacturer
            ));
        }

        // 内存
        text.push_str(&format!("内存: {}\n", detector.memory_info.name));

        // 主板
        text.push_str(&format!(
            "主板: {} {} ({}, {})\n",
            detector.motherboard_info.manufacturer,
            detector.motherboard_info.product_name,
            detector.motherboard_info.chipset,
            detector.motherboard_info.bios_vendor
        ));

        // 显示器
        text.push_str(&format!("显示器: {}\n", detector.monitor_info.name));

        // 硬盘
        let memory_capacity = format_mb_size!(detector.disk_info.total_capacity);
        text.push_str(&format!(
            "主硬盘:({}) {} ({} / {})\n",
            detector.disk_info.disk_type.to_string(),
            detector.disk_info.model,
            &memory_capacity,
            detector.disk_info.disk_type.to_string()
        ));

        // 网卡
        for adapter in detector.network_info.adapters.iter() {
            text.push_str(&format!("网卡: {}\n", adapter.to_string()));
        }

        // 电池
        for battery in detector.battery_info.batteries.iter() {
            text.push_str(&format!(
                "电池: {} {} {} (健康度：{:.0}%)\n",
                battery.vendor,
                battery.name,
                battery.chemistry.to_string(),
                battery.health
            ));
        }

        // 声卡
        if let Some(device) = detector.audio_info.devices.first() {
            text.push_str(&format!("声卡: {}\n", device.name));
        } else {
            text.push_str("声卡: 未检测到\n");
        }

        text
    }

    /// 渲染电脑标题
    fn render_computer_title(&mut self, ui: &mut egui::Ui) {
        let detector = self.detector.lock().expect("硬件检测器互斥锁被污染");
        let system_type = match detector.system_info.computer_type {
            crate::detector::ComputerType::Laptop => "笔记本",
            crate::detector::ComputerType::Desktop => "台式机",
            crate::detector::ComputerType::Unknown => "主机",
        };
        ui.heading(format!(
            "{} {} {}",
            detector.system_info.system_manufacturer,
            detector.system_info.computer_model,
            system_type
        ));
    }

    /// 渲染硬件信息
    fn render_hardware_info(&mut self, ui: &mut egui::Ui) {
        let detector = self.detector.lock().expect("硬件检测器互斥锁被污染");

        egui::Grid::new("hardware_info_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("操作系统:");
                ui.label(&detector.system_info.os_name);
                ui.end_row();

                ui.label("处理器:");
                ui.label(format!(
                    "{}({})",
                    &detector.cpu_info.name, &detector.cpu_info.cores
                ));
                ui.end_row();

                for gpu in detector.gpu_info.gpus.iter() {
                    if gpu.gpu_type != GpuType::DiscreteGpu
                        && gpu.gpu_type != GpuType::IntegratedGpu
                    {
                        continue;
                    }
                    let vram_size_str = format_mb_size!(gpu.vram_size);
                    ui.label(format!("{}:", gpu.gpu_type.to_string()));
                    ui.label(format!(
                        "{} ({}, {})",
                        gpu.description, vram_size_str, gpu.manufacturer
                    ));
                    ui.end_row();
                }

                ui.label("内存:");
                ui.label(&detector.memory_info.name);
                ui.end_row();

                ui.label("主板:");
                ui.label(format!(
                    "{} {} ({}, {})",
                    &detector.motherboard_info.manufacturer,
                    &detector.motherboard_info.product_name,
                    &detector.motherboard_info.chipset,
                    &detector.motherboard_info.bios_vendor
                ));
                ui.end_row();

                ui.label("显示器:");
                ui.label(&detector.monitor_info.name);

                ui.end_row();

                ui.label("主硬盘:");
                ui.label(format!(
                    "{} ({} GB, {})",
                    &detector.disk_info.model,
                    &detector.disk_info.total_capacity / 1024,
                    &detector.disk_info.disk_type.to_string(),
                ));
                ui.end_row();

                for an in detector.network_info.adapters.iter() {
                    ui.label("网卡:");
                    ui.label(an.to_string());
                    ui.end_row();
                }

                for bt in detector.battery_info.batteries.iter() {
                    ui.label("电池:");
                    let label = ui.label(format!(
                        "{} {} {} (健康度：{:.0}%)",
                        bt.vendor,
                        bt.name,
                        bt.chemistry.to_string(),
                        bt.health
                    ));
                    if bt.health > 100.0 {
                        label.on_hover_text("提示：健康度超过 100% 是正常的");
                    }
                    ui.end_row();
                }

                for device in detector.audio_info.devices.iter() {
                    ui.label("声卡:");
                    ui.label(&device.name);
                    ui.end_row();
                }
            });
    }
}

impl eframe::App for HardwareMasterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查进度更新
        if let Some(rx) = &self.progress_rx {
            while let Ok((progress, message)) = rx.try_recv() {
                self.detection_progress = progress;
                self.detection_message = message;
            }
        }

        // 检查检测是否完成
        if let Some(rx) = &self.detection_rx {
            if let Ok(()) = rx.try_recv() {
                self.is_detecting = false;
                self.has_detected = true;
                self.detection_progress = 1.0;
                self.detection_message = "硬件检测完成！".to_string();
                self.detection_rx = None;
                self.progress_rx = None;
            }
        }

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_detecting {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 40.0);
                    ui.heading(format!(
                        "{} ({:.0}%)",
                        &self.detection_message,
                        self.detection_progress * 100.0
                    ));
                    ui.add_space(20.0);
                    ui.spinner();
                });
            } else {
                ui.horizontal(|ui| {
                    self.render_computer_title(ui);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("刷新").clicked() {
                            self.start_refresh();
                        }
                        if ui.button("复制").clicked() {
                            let info_text = self.get_hardware_info_text();
                            ui.ctx().copy_text(info_text);
                        }
                        ui.separator();
                        ui.label(format!("主题: {}", self.theme));
                        if ui.button("切换").clicked() {
                            self.theme = self.theme.next();
                            self.theme.apply(ctx);
                        }
                    });
                });
                ui.add_space(10.0);

                egui::ScrollArea::both().show(ui, |ui| {
                    self.render_hardware_info(ui);
                });
            }
        });
    }
}
