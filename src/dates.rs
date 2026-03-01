// r[fields.dates]

/// Decode a Things 3 bit-packed date integer to YYYY-MM-DD string.
///
/// Format: year in bits 16-26, month in bits 12-15, day in bits 7-11.
pub fn decode_things_date(value: i64) -> Option<String> {
    if value == 0 {
        return None;
    }
    let year = (value >> 16) & 0x7FF;
    let month = (value >> 12) & 0xF;
    let day = (value >> 7) & 0x1F;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// Encode a YYYY-MM-DD date string to a Things 3 bit-packed integer.
#[cfg(test)]
pub fn encode_things_date(date: &str) -> Option<i64> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;
    Some((year << 16) | (month << 12) | (day << 7))
}

/// Convert a Unix timestamp (f64) to YYYY-MM-DD string.
pub fn unix_timestamp_to_date(ts: f64) -> String {
    let secs = ts as i64;
    // Simple conversion without pulling in chrono
    let days = secs / 86400;
    // Days since 1970-01-01
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_to_ymd(days_since_epoch: i64) -> (i64, i64, i64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as i64, d as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[test.date-decoding]

    #[test]
    fn test_decode_known_date() {
        // 132541952 = 2022-06-28 (verified against real Things data)
        assert_eq!(
            decode_things_date(132541952),
            Some("2022-06-28".to_string())
        );
    }

    #[test]
    fn test_decode_zero() {
        assert_eq!(decode_things_date(0), None);
    }

    #[test]
    fn test_roundtrip() {
        let original = encode_things_date("2024-12-25").unwrap();
        assert_eq!(decode_things_date(original), Some("2024-12-25".to_string()));
    }

    #[test]
    fn test_encode_date() {
        let encoded = encode_things_date("2022-06-28").unwrap();
        assert_eq!(encoded, 132541952);
    }

    #[test]
    fn test_various_dates() {
        // Test a range of dates
        for (y, m, d) in [(2020, 1, 1), (2023, 12, 31), (2025, 6, 15), (2000, 2, 29)] {
            let date_str = format!("{y:04}-{m:02}-{d:02}");
            let encoded = encode_things_date(&date_str).unwrap();
            assert_eq!(decode_things_date(encoded), Some(date_str));
        }
    }

    #[test]
    fn test_unix_timestamp_to_date() {
        // 1656374400 = 2022-06-28 00:00:00 UTC
        assert_eq!(unix_timestamp_to_date(1656374400.0), "2022-06-28");
    }
}
