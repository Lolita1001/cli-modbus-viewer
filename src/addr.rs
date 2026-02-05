//! Парсер строк адресов регистров.
//!
//! Поддержка форматов:
//! - "100"
//! - "100,101,105"
//! - "100-110"
//! - "100-105,200,300-310"

use anyhow::{bail, Result};

pub fn parse_addrs(input: &str) -> Result<Vec<u16>> {
    let input = input.trim();
    if input.is_empty() {
        bail!("пустая строка адресов");
    }

    let mut out = Vec::new();
    for raw in input.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            bail!("пустой сегмент в списке адресов");
        }

        if let Some((a, b)) = part.split_once('-') {
            let a = a.trim();
            let b = b.trim();
            if a.is_empty() || b.is_empty() || b.contains('-') {
                bail!("неверный диапазон: {part}");
            }
            let start = parse_u16(a)?;
            let end = parse_u16(b)?;
            if start > end {
                bail!("неверный диапазон (start > end): {part}");
            }
            out.extend(start..=end);
        } else {
            out.push(parse_u16(part)?);
        }
    }

    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn parse_u16(s: &str) -> Result<u16> {
    let n: u32 = s
        .parse()
        .map_err(|_| anyhow::anyhow!("неверное число: {s}"))?;
    if n > u16::MAX as u32 {
        bail!("число вне диапазона u16: {s}");
    }
    Ok(n as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single() {
        assert_eq!(parse_addrs("100").unwrap(), vec![100]);
    }

    #[test]
    fn list() {
        assert_eq!(parse_addrs("100,101,105").unwrap(), vec![100, 101, 105]);
    }

    #[test]
    fn range() {
        assert_eq!(parse_addrs("100-102").unwrap(), vec![100, 101, 102]);
    }

    #[test]
    fn combo_sorted_dedup() {
        assert_eq!(parse_addrs("3-1").is_err(), true);
        assert_eq!(parse_addrs("1-3,2, 5, 4-4").unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn errors() {
        for s in ["", " ", ",1", "1,", "10-", "-10", "1-2-3", "abc", "70000"] {
            assert!(parse_addrs(s).is_err(), "expected error for {s:?}");
        }
    }
}
