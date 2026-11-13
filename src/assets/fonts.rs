use eframe::egui;
use std::fs;
use std::path::Path;

/// Registers crisp Windows/system proportional and monospace fallback fonts so Spanish accents,
/// typography symbols (`÷`, `×`, `•`, `─`, `│`), and ASCII/Box-Drawing diagrams render with zero `□` tofu.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let proportional_candidates = [
        // Windows
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        // macOS
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/HelveticaNeue.ttc",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
        // Linux (Debian/Ubuntu, Fedora/RHEL, Arch, openSUSE)
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
    ];
    for path in proportional_candidates {
        if try_register_font(&mut fonts, "sys_sans", path, egui::FontFamily::Proportional, true) {
            break;
        }
    }

    let mono_candidates = [
        // Windows
        r"C:\Windows\Fonts\consola.ttf",
        r"C:\Windows\Fonts\lucon.ttf",
        // macOS
        "/System/Library/Fonts/SFNSMono.ttf",
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Monaco.ttf",
        // Linux (Debian/Ubuntu, Fedora/RHEL, Arch, openSUSE)
        "/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/dejavu-sans-mono-fonts/DejaVuSansMono.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        "/usr/share/fonts/liberation-mono/LiberationMono-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
    ];
    for path in mono_candidates {
        if try_register_font(&mut fonts, "sys_mono", path, egui::FontFamily::Monospace, true) {
            break;
        }
    }

    // Secondary symbol fallback (Fluent / Segoe UI Symbol / Arial Unicode if present) for box/math symbols
    let symbol_candidates = [
        r"C:\Windows\Fonts\seguisym.ttf",
        "/System/Library/Fonts/Apple Symbols.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    for path in symbol_candidates {
        if try_register_font(&mut fonts, "sys_sym", path, egui::FontFamily::Proportional, false) {
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("sys_sym".to_string());
            break;
        }
    }

    // CJK fallback so Bilibili / Chinese technical keywords (生电, 复制机, 黑科技, 破坏基岩) render crisply
    let cjk_candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\YuGothR.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-sans-cjk-fonts/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    ];
    for path in cjk_candidates {
        if try_register_font(&mut fonts, "sys_cjk", path, egui::FontFamily::Proportional, false) {
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("sys_cjk".to_string());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn try_register_font(
    fonts: &mut egui::FontDefinitions,
    font_name: &str,
    path: &str,
    family: egui::FontFamily,
    prefer_first: bool,
) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    if let Ok(bytes) = fs::read(path) {
        fonts
            .font_data
            .insert(font_name.to_string(), egui::FontData::from_owned(bytes).into());
        let list = fonts.families.entry(family).or_default();
        if prefer_first {
            list.insert(0, font_name.to_string());
        } else {
            list.push(font_name.to_string());
        }
        return true;
    }
    false
}
