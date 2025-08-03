use libc_cfg::CfgItem;
use libc_cfg::search;

use std::io::Write as _;
use std::ops::Not;

use anyhow::Context;
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use regex::RegexSet;

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    libc: Utf8PathBuf,

    filters: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::parse();

    anyhow::ensure!(opt.filters.is_empty().not(), "no filters specified");

    let re = RegexSet::new(&opt.filters)?;
    let ans = search(&opt.libc, &re).with_context(|| "failed to search")?;

    let mut stdout = std::io::stdout().lock();
    for CfgItem { cfg, name } in ans {
        writeln!(stdout, "#[cfg({cfg})]\npub use libc::{name};\n")?;
    }

    Ok(())
}
