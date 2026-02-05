mod addr;
mod cli;
mod modbus;
mod render;

use std::{io::Write, time::Duration};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::cli::Cli;
use crate::modbus::RegType;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse().normalize()?;

    let reqs = build_reqs(&cli)?;
    let host = cli.host.clone();
    let port = cli.port;
    let unit = cli.unit;

    let mut client = modbus::Client::new(cli.host, port, unit, Duration::from_millis(cli.timeout_ms));

    if cli.watch {
        watch_loop(&mut client, &reqs, &host, port, unit, Duration::from_millis(cli.interval_ms))
            .await?;
    } else {
        let mut rows = client.poll(&reqs).await;
        rows.sort_by_key(|r| (r.reg_type, r.address));
        println!("{}", render::render(&rows, &host, port, unit));
    }

    Ok(())
}

fn build_reqs(cli: &Cli) -> Result<Vec<(RegType, Vec<u16>)>> {
    let mut out = Vec::new();
    if let Some(s) = &cli.holding {
        out.push((RegType::Holding, addr::parse_addrs(s)?));
    }
    if let Some(s) = &cli.input {
        out.push((RegType::Input, addr::parse_addrs(s)?));
    }
    if let Some(s) = &cli.coils {
        out.push((RegType::Coils, addr::parse_addrs(s)?));
    }
    if let Some(s) = &cli.discrete {
        out.push((RegType::Discrete, addr::parse_addrs(s)?));
    }
    Ok(out)
}

async fn watch_loop(
    client: &mut modbus::Client,
    reqs: &[(RegType, Vec<u16>)],
    host: &str,
    port: u16,
    unit: u8,
    interval: Duration,
) -> Result<()> {
    loop {
        let mut rows = client.poll(reqs).await;
        rows.sort_by_key(|r| (r.reg_type, r.address));
        let s = render::render(&rows, host, port, unit);

        clear_screen()?;
        print!("{s}");
        std::io::stdout().flush()?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = tokio::time::sleep(interval) => {}
        }
    }
    Ok(())
}

fn clear_screen() -> Result<()> {
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}
