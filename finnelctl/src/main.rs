use anyhow::Result;

mod application;

#[cfg(test)]
mod test;

fn main() -> Result<()> {
    application::run()
}
