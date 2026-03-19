/// ダイアグラムレンダリングのトレイト
pub trait DiagramRenderer {
    /// draw.io XML を受け取り、PNG バイナリを返す
    fn render_to_png(&self, xml: &str, scale: f64) -> anyhow::Result<Vec<u8>>;
}
