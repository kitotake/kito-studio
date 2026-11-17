//! Horodatages UTC sans dépendance externe.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_secs(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

pub fn now_unix_secs() -> i64 {
    unix_secs(SystemTime::now())
}

/// Convertit un nombre de jours depuis 1970-01-01 en (année, mois, jour).
/// Algorithme « civil_from_days » de Howard Hinnant.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn parts(secs: i64) -> (i64, u32, u32, i64, i64, i64) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    (y, m, d, rem / 3_600, (rem % 3_600) / 60, rem % 60)
}

/// `2026-11-15T14:32:05Z`
pub fn format_iso(secs: i64) -> String {
    let (y, m, d, hh, mm, ss) = parts(secs);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// `20261115-143205`, utilisable dans un nom de fichier.
pub fn compact_stamp(secs: i64) -> String {
    let (y, m, d, hh, mm, ss) = parts(secs);
    format!("{y:04}{m:02}{d:02}-{hh:02}{mm:02}{ss:02}")
}

pub fn now_iso() -> String {
    format_iso(now_unix_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_iso(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(format_iso(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_iso(1_709_164_799), "2024-02-28T23:59:59Z");
        assert_eq!(format_iso(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn compact_stamp_is_filename_safe() {
        assert_eq!(compact_stamp(1_700_000_000), "20231114-221320");
    }
}
