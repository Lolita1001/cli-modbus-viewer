//! Modbus TCP: подключение и чтение регистров.

use std::time::Duration;

use tokio_modbus::{
    client::{tcp, Context},
    prelude::{Reader, SlaveContext},
    ExceptionCode, Slave,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegType {
    Holding,
    Input,
    Coils,
    Discrete,
}

impl RegType {
    pub fn short(self) -> &'static str {
        match self {
            Self::Holding => "HR",
            Self::Input => "IR",
            Self::Coils => "CO",
            Self::Discrete => "DI",
        }
    }

    fn max_qty(self) -> u16 {
        match self {
            Self::Holding | Self::Input => 125,
            Self::Coils | Self::Discrete => 2000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub address: u16,
    pub reg_type: RegType,
    pub cell: Cell,
}

#[derive(Debug, Clone)]
pub enum Cell {
    Ok { raw: u16, bool: Option<bool> },
    Err(CellErr),
}

#[derive(Debug, Clone)]
pub enum CellErr {
    Timeout,
    Offline,
    NotAvailable,
    ModbusException(u8),
}

pub struct Client {
    host: String,
    port: u16,
    unit: u8,
    timeout: Duration,
    ctx: Option<Context>,
}

impl Client {
    pub fn new(host: String, port: u16, unit: u8, timeout: Duration) -> Self {
        Self {
            host,
            port,
            unit,
            timeout,
            ctx: None,
        }
    }

    pub async fn poll(&mut self, reqs: &[(RegType, Vec<u16>)]) -> Vec<Row> {
        if self.ctx.is_none() && self.connect().await.is_err() {
            return offline_rows(reqs);
        }

        let mut out = Vec::new();
        let mut offline = false;
        for (ty, addrs) in reqs {
            if addrs.is_empty() {
                continue;
            }
            if offline {
                out.extend(addrs.iter().map(|&a| Row {
                    address: a,
                    reg_type: *ty,
                    cell: Cell::Err(CellErr::Offline),
                }));
                continue;
            }

            let ctx = self.ctx.as_mut().expect("connected");
            let (rows, had_offline) = poll_type(ctx, *ty, addrs, self.timeout).await;
            out.extend(rows);
            if had_offline {
                offline = true;
                self.ctx = None;
            }
        }
        out
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        let mut last_err: Option<anyhow::Error> = None;
        for addr in tokio::net::lookup_host((self.host.as_str(), self.port)).await? {
            match tokio::time::timeout(self.timeout, tcp::connect(addr)).await {
                Ok(Ok(mut ctx)) => {
                    ctx.set_slave(Slave(self.unit));
                    self.ctx = Some(ctx);
                    return Ok(());
                }
                Ok(Err(e)) => last_err = Some(e.into()),
                Err(e) => last_err = Some(e.into()),
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("не удалось разрешить адрес")))
    }
}

fn offline_rows(reqs: &[(RegType, Vec<u16>)]) -> Vec<Row> {
    let mut out = Vec::new();
    for (ty, addrs) in reqs {
        out.extend(addrs.iter().map(|&a| Row {
            address: a,
            reg_type: *ty,
            cell: Cell::Err(CellErr::Offline),
        }));
    }
    out
}

async fn poll_type(
    ctx: &mut Context,
    ty: RegType,
    addrs: &[u16],
    timeout: Duration,
) -> (Vec<Row>, bool) {
    let segs = contiguous_segments(addrs, ty.max_qty());
    let mut out = Vec::with_capacity(addrs.len());
    let mut offline = false;

    for (start, qty) in segs {
        if offline {
            for i in 0..qty {
                let a = start + i;
                out.push(Row {
                    address: a,
                    reg_type: ty,
                    cell: Cell::Err(CellErr::Offline),
                });
            }
            continue;
        }

        let (cells, had_offline) = read_range_cells(ctx, ty, start, qty, timeout).await;
        for (i, cell) in cells.into_iter().enumerate() {
            let addr = start + i as u16;
            out.push(Row {
                address: addr,
                reg_type: ty,
                cell,
            });
        }
        offline |= had_offline;
    }

    (out, offline)
}

#[derive(Debug)]
enum RangeRead {
    Ok(Vec<u16>),
    Timeout,
    Exception(ExceptionCode),
    Offline,
}

async fn read_range(
    ctx: &mut Context,
    ty: RegType,
    start: u16,
    qty: u16,
    timeout: Duration,
) -> RangeRead {
    let res = match ty {
        RegType::Holding => tokio::time::timeout(timeout, ctx.read_holding_registers(start, qty))
            .await
            .map(|r| r.map(|rr| rr.map(|v| v))),
        RegType::Input => tokio::time::timeout(timeout, ctx.read_input_registers(start, qty))
            .await
            .map(|r| r.map(|rr| rr.map(|v| v))),
        RegType::Coils => tokio::time::timeout(timeout, ctx.read_coils(start, qty))
            .await
            .map(|r| r.map(|rr| rr.map(|v| v.into_iter().map(u16::from).collect()))),
        RegType::Discrete => tokio::time::timeout(timeout, ctx.read_discrete_inputs(start, qty))
            .await
            .map(|r| r.map(|rr| rr.map(|v| v.into_iter().map(u16::from).collect()))),
    };

    match res {
        Err(_) => RangeRead::Timeout,
        Ok(Err(_)) => RangeRead::Offline,
        Ok(Ok(Err(code))) => RangeRead::Exception(code),
        Ok(Ok(Ok(values))) => RangeRead::Ok(values),
    }
}

async fn read_range_cells(
    ctx: &mut Context,
    ty: RegType,
    start: u16,
    qty: u16,
    timeout: Duration,
) -> (Vec<Cell>, bool) {
    let mut out = vec![Cell::Err(CellErr::Offline); qty as usize];
    let mut stack = vec![(start, qty, 0usize)];
    let mut offline = false;

    while let Some((s, q, off)) = stack.pop() {
        if offline {
            for i in 0..q as usize {
                out[off + i] = Cell::Err(CellErr::Offline);
            }
            continue;
        }

        match read_range(ctx, ty, s, q, timeout).await {
            RangeRead::Ok(values) => {
                for (i, v) in values.into_iter().enumerate() {
                    out[off + i] = ok_cell(ty, v);
                }
            }
            RangeRead::Timeout => {
                for i in 0..q as usize {
                    out[off + i] = Cell::Err(CellErr::Timeout);
                }
            }
            RangeRead::Offline => {
                offline = true;
                for i in 0..q as usize {
                    out[off + i] = Cell::Err(CellErr::Offline);
                }
            }
            RangeRead::Exception(code) => {
                if code != ExceptionCode::IllegalDataAddress {
                    for i in 0..q as usize {
                        out[off + i] = Cell::Err(CellErr::ModbusException(u8::from(code)));
                    }
                    continue;
                }
                if q == 1 {
                    out[off] = Cell::Err(CellErr::NotAvailable);
                    continue;
                }

                let left_q = q / 2;
                let right_q = q - left_q;
                stack.push((s + left_q, right_q, off + left_q as usize));
                stack.push((s, left_q, off));
            }
        }
    }

    (out, offline)
}

fn ok_cell(ty: RegType, raw: u16) -> Cell {
    let b = matches!(ty, RegType::Coils | RegType::Discrete).then_some(raw != 0);
    Cell::Ok { raw, bool: b }
}

fn contiguous_segments(addrs: &[u16], max_qty: u16) -> Vec<(u16, u16)> {
    let mut segs = Vec::new();
    let mut i = 0;
    while i < addrs.len() {
        let start = addrs[i];
        let mut qty = 1u16;
        i += 1;
        while i < addrs.len() && qty < max_qty {
            let prev = addrs[i - 1];
            let next = addrs[i];
            if prev != u16::MAX && next == prev + 1 {
                qty += 1;
                i += 1;
            } else {
                break;
            }
        }
        segs.push((start, qty));
    }
    segs
}
