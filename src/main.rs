mod adapter;
mod domain;
mod usecase;

fn main() -> anyhow::Result<()> {
    adapter::cli::run()
}
