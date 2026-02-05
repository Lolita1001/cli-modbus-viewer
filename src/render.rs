//! Рендер таблицы и форматирование значений.

use chrono::Local;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};

use crate::modbus::{Cell, CellErr, RegType, Row};

pub fn render(rows: &[Row], host: &str, port: u16, unit: u8) -> String {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_content_arrangement(ContentArrangement::Dynamic);
    t.set_header(vec![
        "Address", "Type", "Hex", "UInt16", "Int16", "Binary", "Bool",
    ]);

    for r in rows {
        let (hex, u16s, i16s, bin, bools) = match &r.cell {
            Cell::Ok { raw, bool } => (
                format!("0x{raw:04X}"),
                raw.to_string(),
                (*raw as i16).to_string(),
                format!("{raw:016b}"),
                bool.map(|b| b.to_string()).unwrap_or_else(|| "-".into()),
            ),
            Cell::Err(e) => {
                let s = err_text(e);
                let b = if matches!(r.reg_type, RegType::Coils | RegType::Discrete) {
                    s.clone()
                } else {
                    "-".into()
                };
                (s.clone(), s.clone(), s.clone(), s, b)
            }
        };

        t.add_row(vec![
            r.address.to_string(),
            r.reg_type.short().to_string(),
            hex,
            u16s,
            i16s,
            bin,
            bools,
        ]);
    }

    let updated = Local::now().format("%Y-%m-%d %H:%M:%S");
    format!("{t}\nHost: {host}:{port} | Unit: {unit} | Updated: {updated}")
}

fn err_text(e: &CellErr) -> String {
    match e {
        CellErr::Timeout => "TIMEOUT".into(),
        CellErr::Offline => "OFFLINE".into(),
        CellErr::NotAvailable => "N/A".into(),
        CellErr::ModbusException(code) => format!("ERR:{code}"),
    }
}
