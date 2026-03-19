mod adapter;
mod domain;
#[cfg(feature = "render")]
mod infra;
mod usecase;

fn main() -> anyhow::Result<()> {
    adapter::cli::run()
}
