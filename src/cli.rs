//! CLI parsing (clap) и валидация аргументов.

use anyhow::{bail, Result};
use clap::{ArgAction, Parser};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, disable_help_flag = true)]
pub struct Cli {
    /// Показать справку
    #[arg(long, short = '?', action = ArgAction::Help)]
    help: Option<bool>,

    /// Адрес устройства (IP или hostname)
    #[arg(short = 'h', long)]
    pub host: String,

    /// TCP порт
    #[arg(short = 'p', long, default_value_t = 502)]
    pub port: u16,

    /// Unit ID / Slave ID
    #[arg(short = 'u', long, default_value_t = 1)]
    pub unit: u8,

    /// Таймаут в мс (подключение и каждый запрос)
    #[arg(short = 't', long = "timeout", default_value_t = 1000, value_name = "MS")]
    pub timeout_ms: u64,

    /// Holding Registers (FC 03)
    #[arg(long, value_name = "ADDRS")]
    pub holding: Option<String>,

    /// Input Registers (FC 04)
    #[arg(long, value_name = "ADDRS")]
    pub input: Option<String>,

    /// Coils (FC 01)
    #[arg(long, value_name = "ADDRS")]
    pub coils: Option<String>,

    /// Discrete Inputs (FC 02)
    #[arg(long, value_name = "ADDRS")]
    pub discrete: Option<String>,

    /// Непрерывный опрос с обновлением
    #[arg(short = 'w', long)]
    pub watch: bool,

    /// Интервал опроса в watch-режиме (мс)
    #[arg(long = "interval", default_value_t = 1000, value_name = "MS")]
    pub interval_ms: u64,

    /// Адреса (если типы регистров не заданы, считается holding)
    #[arg(value_name = "ADDRS")]
    pub addrs: Option<String>,
}

impl Cli {
    pub fn normalize(mut self) -> Result<Self> {
        let any_typed = self.holding.is_some()
            || self.input.is_some()
            || self.coils.is_some()
            || self.discrete.is_some();

        if any_typed {
            if self.addrs.is_some() {
                bail!("нельзя использовать позиционные ADDRS вместе с --holding/--input/--coils/--discrete");
            }
        } else {
            self.holding = self.addrs.take();
        }

        for (name, opt) in [
            ("holding", &self.holding),
            ("input", &self.input),
            ("coils", &self.coils),
            ("discrete", &self.discrete),
        ] {
            if let Some(s) = opt
                && s.trim().is_empty()
            {
                bail!("пустая строка адресов для --{name}");
            }
        }

        if self.holding.is_none() && self.input.is_none() && self.coils.is_none() && self.discrete.is_none() {
            bail!("не заданы адреса регистров");
        }
        if self.timeout_ms == 0 {
            bail!("--timeout должен быть > 0");
        }
        if self.watch && self.interval_ms == 0 {
            bail!("--interval должен быть > 0");
        }

        Ok(self)
    }
}
