use clap::{Parser, Subcommand};
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "drawio-tools",
    version,
    about = "Draw.io PNG/XML bidirectional conversion CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract draw.io XML from an embedded PNG
    Extract {
        /// Input PNG file path
        input: String,
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Embed draw.io XML into an existing PNG
    Embed {
        /// Input .drawio XML file path
        xml: String,
        /// Input PNG file path
        png: String,
        /// Output PNG file path
        #[arg(short, long)]
        output: String,
    },
    /// Render draw.io XML to PNG with embedded XML
    #[cfg(feature = "render")]
    Export {
        /// Input .drawio XML file path
        input: String,
        /// Output PNG file path
        #[arg(short, long, default_value = "output.png")]
        output: String,
        /// Scale factor
        #[arg(short, long, default_value = "1.0")]
        scale: f64,
        /// Chromium binary path (auto-detect if omitted)
        #[arg(long)]
        chromium_path: Option<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Extract { input, output } => {
            let xml = crate::usecase::extract::extract_xml_from_png(&input)?;
            match output {
                Some(path) => std::fs::write(&path, &xml)?,
                None => std::io::stdout().write_all(xml.as_bytes())?,
            }
        }
        Commands::Embed { xml, png, output } => {
            let result = crate::usecase::embed::embed_xml_into_png(&xml, &png)?;
            std::fs::write(&output, &result)?;
        }
        #[cfg(feature = "render")]
        Commands::Export {
            input,
            output,
            scale,
            chromium_path,
        } => {
            let xml = std::fs::read_to_string(&input)?;
            let renderer = crate::infra::chrome_renderer::ChromeRenderer::new(chromium_path);
            let result = crate::usecase::export::export_drawio_to_png(&renderer, &xml, scale)?;
            std::fs::write(&output, &result)?;
            eprintln!(
                "Exported to {} ({} bytes, XML embedded)",
                output,
                result.len()
            );
        }
    }
    Ok(())
}
