//! Host-independent checks for the proposed canonical Markdown record format.
//! This is not a graph reader, Markdown index, or substitute for the host parser.

use std::collections::BTreeMap;

const MAX_MINOR_UNITS: i64 = 99_999_999_999;

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub kind: String,
    pub minor_units: i64,
    pub category: Option<String>,
}

pub fn amount_minor_units(raw: &str) -> Result<i64, String> {
    let mut parts = raw.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    if whole.is_empty() || !whole.bytes().all(|c| c.is_ascii_digit()) || parts.next().is_some() {
        return Err("amount must be a positive plain decimal, without signs or separators".into());
    }
    let fractional_minor = match fraction {
        None => 0,
        Some(s) if (1..=2).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_digit()) => {
            let n = s.parse::<i64>().map_err(|_| "invalid amount")?;
            if s.len() == 1 {
                n * 10
            } else {
                n
            }
        }
        _ => return Err("CNY amounts support at most two decimal places".into()),
    };
    let whole = whole
        .parse::<i64>()
        .map_err(|_| "amount exceeds the supported range")?;
    let minor = whole
        .checked_mul(100)
        .and_then(|n| n.checked_add(fractional_minor))
        .ok_or("amount exceeds the supported range")?;
    if !(1..=MAX_MINOR_UNITS).contains(&minor) {
        return Err("amount must be greater than zero and within the supported range".into());
    }
    Ok(minor)
}

pub fn valid_calendar_time(s: &str) -> bool {
    if s.len() != 14 || !s.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let number = |start, end| s[start..end].parse::<u32>().unwrap_or(0);
    let year = number(0, 4);
    let month = number(4, 6);
    let day = number(6, 8);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year > 0
        && (1..=days).contains(&day)
        && number(8, 10) < 24
        && number(10, 12) < 60
        && number(12, 14) < 60
}

pub fn valid_offset(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 6
        || !matches!(b[0], b'+' | b'-')
        || b[3] != b':'
        || ![b[1], b[2], b[4], b[5]].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let hours = (b[1] - b'0') * 10 + b[2] - b'0';
    let minutes = (b[4] - b'0') * 10 + b[5] - b'0';
    minutes < 60 && hours <= 14 && (hours != 14 || minutes == 0) && s != "-00:00"
}

fn page_reference(s: &str) -> bool {
    s.strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
        .is_some_and(|name| !name.trim().is_empty() && !name.contains(['[', ']', '\n', '\r']))
}

fn block_reference(s: &str) -> bool {
    let Some(id) = s.strip_prefix("((").and_then(|s| s.strip_suffix("))")) else {
        return false;
    };
    id.len() == 36
        && id.bytes().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

/// Explicitly limited diagnostic: only canonical direct property lines outside
/// fenced examples, from the single block consented to by a user command.
pub fn validate_canonical_block(raw: &str) -> Result<Transaction, String> {
    if raw.len() > 128 * 1024 {
        return Err("block exceeds the diagnostic input bound".into());
    }
    let mut properties = BTreeMap::new();
    let mut fence: Option<(char, usize)> = None;
    for line in raw.lines() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        if indent > 3 {
            continue;
        }
        let line = line.trim_start_matches(' ');
        if let Some((ch, count)) = fence {
            let closing = line.chars().take_while(|c| *c == ch).count();
            if closing >= count && line[closing..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if let Some(ch @ ('`' | '~')) = line.chars().next() {
            let count = line.chars().take_while(|c| *c == ch).count();
            if count >= 3 && (ch != '`' || !line[count..].contains('`')) {
                fence = Some((ch, count));
                continue;
            }
        }
        let Some((key, value)) = line.split_once("::") else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        if !key.starts_with("ledger-") {
            continue;
        }
        if key.len() > 64 {
            return Err("ledger property name exceeds the diagnostic bound".into());
        }
        if properties
            .insert(key.clone(), value.trim().to_owned())
            .is_some()
        {
            return Err(format!("duplicate property: {key}"));
        }
    }
    const ALLOWED: &[&str] = &[
        "ledger-kind",
        "ledger-amount",
        "ledger-currency",
        "ledger-category",
        "ledger-at",
        "ledger-timezone",
        "ledger-account",
        "ledger-project",
        "ledger-recorded-at",
        "ledger-to-account",
        "ledger-refund-of",
    ];
    for key in properties.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            return Err(format!("unsupported ledger property: {key}"));
        }
    }
    let required = |name: &str| {
        properties
            .get(name)
            .filter(|s| !s.is_empty())
            .map(String::as_str)
            .ok_or_else(|| format!("missing {name}"))
    };
    let kind = required("ledger-kind")?;
    if !["expense", "income", "refund", "transfer"].contains(&kind) {
        return Err("unsupported ledger-kind".into());
    }
    if required("ledger-currency")? != "CNY" {
        return Err(
            "this diagnostic supports CNY only; no currency conversion is performed".into(),
        );
    }
    let minor_units = amount_minor_units(required("ledger-amount")?)?;
    if !valid_calendar_time(required("ledger-at")?) {
        return Err("ledger-at must be a valid yyyyMMddHHmmss calendar time".into());
    }
    if !valid_offset(required("ledger-timezone")?) {
        return Err("ledger-timezone must be a known UTC offset such as +08:00".into());
    }
    let account = required("ledger-account")?;
    if !page_reference(account) {
        return Err("ledger-account must be one page reference".into());
    }
    let category = if kind == "transfer" {
        let destination = required("ledger-to-account")?;
        if !page_reference(destination) || destination.to_lowercase() == account.to_lowercase() {
            return Err("a transfer requires a different destination account".into());
        }
        if properties.contains_key("ledger-category") {
            return Err("own-account transfers must not have a consumption category".into());
        }
        None
    } else {
        let category = required("ledger-category")?;
        if !page_reference(category) {
            return Err("ledger-category must be one page reference".into());
        }
        if properties.contains_key("ledger-to-account") {
            return Err("ledger-to-account is valid only for transfers".into());
        }
        Some(category.to_owned())
    };
    if kind == "refund" {
        if !block_reference(required("ledger-refund-of")?) {
            return Err("refunds must reference an original block UUID".into());
        }
    } else if properties.contains_key("ledger-refund-of") {
        return Err("ledger-refund-of is valid only for refunds".into());
    }
    if let Some(recorded) = properties.get("ledger-recorded-at") {
        if !valid_calendar_time(recorded) {
            return Err("invalid ledger-recorded-at".into());
        }
    }
    if properties
        .get("ledger-project")
        .is_some_and(|p| !page_reference(p))
    {
        return Err("ledger-project must be one page reference".into());
    }
    Ok(Transaction {
        kind: kind.to_owned(),
        minor_units,
        category,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const RECORD: &str = "Lunch\nledger-kind:: expense\nledger-amount:: 28.00\nledger-currency:: CNY\nledger-category:: [[Ledger categories/Food/Meals]]\nledger-at:: 20261026123000\nledger-timezone:: +08:00\nledger-account:: [[Wallet]]";

    #[test]
    fn amounts_are_exact() {
        assert_eq!(amount_minor_units("0.1"), Ok(10));
        assert_eq!(amount_minor_units("0.20"), Ok(20));
        assert_eq!(
            amount_minor_units("0.1").unwrap() + amount_minor_units("0.20").unwrap(),
            30
        );
        assert_eq!(amount_minor_units("999999999.99"), Ok(MAX_MINOR_UNITS));
    }
    #[test]
    fn invalid_amounts_are_not_coerced() {
        for value in [
            "0",
            "-1",
            "+1",
            "1e2",
            "NaN",
            "1,000",
            "1.001",
            "1.",
            ".5",
            " 1",
            "1000000000",
            "9999999999999999999999999999999",
            "１",
        ] {
            assert!(amount_minor_units(value).is_err(), "accepted {value}");
        }
    }
    #[test]
    fn dates_are_real_calendar_dates() {
        for value in ["20240229235959", "20000229000000", "20261026123000"] {
            assert!(valid_calendar_time(value));
        }
        for value in [
            "20260229000000",
            "19000229000000",
            "20261301000000",
            "20260431000000",
            "20261026240000",
            "20261026126000",
            "00000101000000",
            "20261026",
            "２０２６１０２６１２３０００",
        ] {
            assert!(!valid_calendar_time(value), "accepted {value}");
        }
    }
    #[test]
    fn offset_bounds_and_unknown_offset_are_checked() {
        for value in ["+08:00", "-05:30", "+14:00", "+00:00"] {
            assert!(valid_offset(value));
        }
        for value in [
            "+14:01",
            "+15:00",
            "+08:60",
            "-00:00",
            "Asia/Singapore",
            "Z",
        ] {
            assert!(!valid_offset(value));
        }
    }
    #[test]
    fn validates_canonical_record_without_editing_source() {
        assert_eq!(validate_canonical_block(RECORD).unwrap().minor_units, 2800);
        assert_eq!(
            validate_canonical_block(&format!("{RECORD}\nother:: keep me\nAdditional note"))
                .unwrap()
                .minor_units,
            2800
        );
    }
    #[test]
    fn rejects_missing_duplicate_unknown_and_multicurrency_fields() {
        for raw in [
            RECORD.replace("ledger-at:: 20261026123000", ""),
            format!("{RECORD}\nLedger-Amount:: 2"),
            format!("{RECORD}\nledger-mystery:: 2"),
            RECORD.replace("CNY", "USD"),
        ] {
            assert!(validate_canonical_block(&raw).is_err());
        }
    }
    #[test]
    fn examples_in_code_fences_are_not_transactions() {
        for fence in ["```", "~~~~"] {
            assert!(validate_canonical_block(&format!("{fence}\n{RECORD}\n{fence}")).is_err());
        }
        assert!(validate_canonical_block(&format!("```\n```not-a-close\n{RECORD}\n```")).is_err());
        assert_eq!(
            validate_canonical_block(&format!("~~~\nledger-amount:: 99\n~~~\n{RECORD}"))
                .unwrap()
                .minor_units,
            2800
        );
    }
    #[test]
    fn transfers_require_distinct_accounts_and_no_expense_category() {
        let transfer = RECORD
            .replace("expense", "transfer")
            .replace("ledger-category:: [[Ledger categories/Food/Meals]]\n", "")
            + "\nledger-to-account:: [[Bank]]";
        assert_eq!(validate_canonical_block(&transfer).unwrap().category, None);
        assert!(validate_canonical_block(&transfer.replace("[[Bank]]", "[[wallet]]")).is_err());
        assert!(validate_canonical_block(&(transfer + "\nledger-category:: [[Food]]")).is_err());
    }
    #[test]
    fn refunds_need_an_original_identity() {
        let refund = RECORD.replace("expense", "refund");
        assert!(validate_canonical_block(&refund).is_err());
        assert!(validate_canonical_block(
            &(refund.clone() + "\nledger-refund-of:: ((4a0e7a01-1479-4cb6-b9c8-534635837c2a))")
        )
        .is_ok());
        assert!(
            validate_canonical_block(&(refund + "\nledger-refund-of:: ((not-a-uuid))")).is_err()
        );
    }
}
