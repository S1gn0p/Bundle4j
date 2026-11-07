// 隐藏控制台黑框
#![windows_subsystem = "windows"]

use eframe::egui;
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom, Cursor};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;

fn main() -> Result<(), io::Error> {
    let current_exe = std::env::current_exe()?;

    // 检查自己尾部是否有 "JPE1" 标记
    if is_packed_exe(&current_exe)? {
        // 如果有标记，说明自己是最终生成的文件，进入 Stub（运行）模式
        run_stub(&current_exe)?;
    } else {
        // 如果没有标记，说明自己是打包器，进入 GUI（打包）模式
        run_packer_gui().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    }

    Ok(())
}

// ================= 核心检测逻辑 =================
fn is_packed_exe(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    if size < 20 { return Ok(false); } // 文件太小肯定没打包

    // 读取最后 20 个字节检查魔数
    file.seek(SeekFrom::End(-20))?;
    let mut trailer = [0u8; 20];
    file.read_exact(&mut trailer)?;
    Ok(&trailer[16..20] == b"JPE1")
}

// ================= Stub 模式（运行） =================
fn run_stub(current_exe: &Path) -> io::Result<()> {
    let mut file = File::open(current_exe)?;
    let file_size = file.metadata()?.len();

    file.seek(SeekFrom::End(-20))?;
    let mut trailer = [0u8; 20];
    file.read_exact(&mut trailer)?;

    let exe_len = u64::from_le_bytes(trailer[0..8].try_into().unwrap());
    let zip_len = u64::from_le_bytes(trailer[8..16].try_into().unwrap());

    let exe_start = file_size - 20 - zip_len - exe_len;
    let zip_start = file_size - 20 - zip_len;

    let temp_dir = std::env::temp_dir().join("RustJavaApp_Runtime");
    fs::create_dir_all(&temp_dir)?;

    let target_exe = temp_dir.join("app.exe");
    let target_zip = temp_dir.join("jre.zip");

    // 提取原 Java exe
    file.seek(SeekFrom::Start(exe_start))?;
    let mut exe_buf = vec![0; exe_len as usize];
    file.read_exact(&mut exe_buf)?;
    fs::write(&target_exe, &exe_buf)?;

    // 提取并解压 JRE
    file.seek(SeekFrom::Start(zip_start))?;
    let mut zip_buf = vec![0; zip_len as usize];
    file.read_exact(&mut zip_buf)?;
    fs::write(&target_zip, &zip_buf)?;

    let jre_dir = temp_dir.join("jre");
    if !jre_dir.exists() {
        let mut archive = zip::ZipArchive::new(File::open(&target_zip)?)?;
        archive.extract(&jre_dir)?;
    }
    let _ = fs::remove_file(&target_zip);

    // 启动原程序
    let mut child = std::process::Command::new(&target_exe)
        .current_dir(&temp_dir)
        .spawn()?;

    let _ = child.wait();
    Ok(())
}

// ================= GUI 模式（打包） =================
fn run_packer_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 350.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Java 单文件打包器",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(PackerApp::default())
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert("my_font".to_owned(), egui::FontData::from_owned(font_data));
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "my_font".to_owned());
        ctx.set_fonts(fonts);
    }
}

#[derive(Default)]
struct PackerApp {
    exe_path: Option<PathBuf>,
    jre_dir: Option<PathBuf>,
    status_msg: String,
    is_success: bool,
}

impl eframe::App for PackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("将 app.exe 和 jre 文件夹拖拽到此处");

            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if !dropped_files.is_empty() {
                for file in dropped_files {
                    if let Some(path) = file.path {
                        if path.is_dir() {
                            self.jre_dir = Some(path);
                        } else if path.extension().map_or(false, |ext| ext == "exe") {
                            self.exe_path = Some(path);
                        }
                    }
                }
            }

            ui.add_space(20.0);
            ui.group(|ui| {
                ui.label(format!("1. 目标 EXE: {:?}", self.exe_path.as_deref().unwrap_or(Path::new("未选择"))));
                ui.label(format!("2. JRE 文件夹: {:?}", self.jre_dir.as_deref().unwrap_or(Path::new("未选择"))));
            });

            ui.add_space(20.0);

            if ui.button("开始生成单文件").clicked() {
                if let (Some(exe), Some(jre)) = (&self.exe_path, &self.jre_dir) {
                    self.status_msg = "正在打包，请稍候...".to_string();
                    self.is_success = false;

                    match build_single_exe(exe, jre) {
                        Ok(out) => {
                            self.status_msg = format!("成功生成: {:?}", out);
                            self.is_success = true;
                        }
                        Err(e) => {
                            self.status_msg = format!("打包失败: {}", e);
                        }
                    }
                } else {
                    self.status_msg = "请先拖入 EXE 和 JRE 文件夹！".to_string();
                    self.is_success = false;
                }
            }

            ui.add_space(20.0);
            let color = if self.is_success { egui::Color32::GREEN } else { egui::Color32::RED };
            ui.colored_label(color, &self.status_msg);
        });
    }
}

fn build_single_exe(exe_path: &Path, jre_dir: &Path) -> io::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let output_path = exe_path.with_file_name("SingleApp.exe");
    let mut out_file = File::create(&output_path)?;

    // 🌟 关键修改：直接把自己（当前的 packer.exe）作为开头写入！
    let mut self_exe = File::open(&current_exe)?;
    io::copy(&mut self_exe, &mut out_file)?;

    let exe_size = fs::metadata(exe_path)?.len();
    let mut real_exe = File::open(exe_path)?;
    io::copy(&mut real_exe, &mut out_file)?;

    let mut zip_buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buffer);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(jre_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let name = path.strip_prefix(jre_dir).unwrap();
                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                io::copy(&mut f, &mut zip)?;
            }
        }
        zip.finish()?;
    }
    let zip_data = zip_buffer.into_inner();
    let zip_size = zip_data.len() as u64;

    out_file.write_all(&zip_data)?;
    out_file.write_all(&exe_size.to_le_bytes())?;
    out_file.write_all(&zip_size.to_le_bytes())?;
    out_file.write_all(b"JPE1")?;

    Ok(output_path)
}