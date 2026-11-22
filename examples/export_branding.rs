//! Reproduces the branding PNGs with the project's existing SVG renderer.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;

use resvg::{tiny_skia, usvg};
use serde::Deserialize;

#[derive(Deserialize)]
struct Export {
    source: String,
    output: String,
    width: u32,
    height: u32,
    #[serde(default)]
    template: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pack = root.join("assets/branding");
    let mut options = usvg::Options {
        resources_dir: Some(pack.clone()),
        ..Default::default()
    };
    for name in ["Inter-Regular.ttf", "Inter-SemiBold.ttf", "Inter-Bold.ttf"] {
        options
            .fontdb_mut()
            .load_font_file(root.join("assets/fonts").join(name))?;
    }
    let exports: Vec<Export> = serde_json::from_slice(&std::fs::read(pack.join("exports.json"))?)?;
    let sources: BTreeSet<_> = exports
        .iter()
        .map(|export| pack.join(&export.source))
        .collect();
    for path in sources {
        let data = std::fs::read_to_string(&path)?;
        let tree = usvg::Tree::from_data(data.as_bytes(), &options)?;
        if data.contains("<text") {
            // Outlined lettering keeps the distributed SVG independent of installed fonts.
            let outlined = tree.to_string(&usvg::WriteOptions::default());
            let outlined = outlined.replacen("<svg ", "<svg role=\"img\" aria-label=\"Abcom\" ", 1);
            if outlined.contains("<text") {
                return Err("SVG lettering was not converted to paths".into());
            }
            std::fs::write(&path, outlined)?;
        }
    }
    for export in exports {
        let tree = usvg::Tree::from_data(&std::fs::read(pack.join(&export.source))?, &options)?;
        let scale = (export.width as f32 / tree.size().width())
            .min(export.height as f32 / tree.size().height());
        let x = (export.width as f32 - tree.size().width() * scale) / 2.0;
        let y = (export.height as f32 - tree.size().height() * scale) / 2.0;
        let transform = tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, x, y);
        let mut pixmap =
            tiny_skia::Pixmap::new(export.width, export.height).ok_or("invalid PNG dimensions")?;
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let pixels = pixmap.data().as_chunks::<4>().0;
        if !pixels.iter().any(|pixel| pixel[3] > 0) {
            return Err(format!("empty render: {}", export.output).into());
        }
        let social_preview = export.source.starts_with("social-preview");
        if !export.source.starts_with("preview")
            && !social_preview
            && !pixels.iter().any(|pixel| pixel[3] == 0)
        {
            return Err(format!("missing transparent background: {}", export.output).into());
        }
        if social_preview
            && (export.width != 1280
                || export.height != 640
                || pixels.iter().any(|pixel| pixel[3] != 255))
        {
            return Err("social previews must be opaque 1280x640 images".into());
        }
        if export.template
            && pixels
                .iter()
                .any(|pixel| pixel[3] > 0 && pixel[..3] != [0, 0, 0])
        {
            return Err("a macOS template must be a black alpha mask".into());
        }
        let output = pack.join(&export.output);
        std::fs::create_dir_all(output.parent().ok_or("missing output directory")?)?;
        pixmap.save_png(&output)?;
        if social_preview && std::fs::metadata(output)?.len() >= 1_000_000 {
            return Err("GitHub social previews must stay below 1 MB".into());
        }
    }
    Ok(())
}
