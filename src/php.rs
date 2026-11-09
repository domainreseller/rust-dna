//! Small re-implementations of the PHP semantics php-dna relies on.
//!
//! php-dna works on loosely typed PHP arrays. To return exactly the same
//! values, this port reproduces the handful of PHP rules that shape them:
//! `??` (missing *or* null), `empty()`, `(int)` / `(string)` casts, float to
//! string conversion, `number_format()`, `http_build_query()`, `trim()`,
//! `strtotime()` for the ISO dates the gateway sends, and `is_numeric()`.

use serde_json::{Map, Value};

/// `$value[$key]` with PHP `??` semantics: `None` when the key is missing or
/// the value is `null`. Works on objects and on lists (numeric keys).
pub fn get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let found = match value {
        Value::Object(map) => map.get(key),
        Value::Array(list) => key.parse::<usize>().ok().and_then(|i| list.get(i)),
        _ => None,
    };
    found.filter(|v| !v.is_null())
}

/// Nested `$value[$a][$b]...` with `??` semantics.
pub fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = get(current, key)?;
    }
    Some(current)
}

/// PHP `is_array()`: a JSON list or object.
pub fn is_array(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

/// PHP truthiness (`(bool) $value`).
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !(s.is_empty() || s == "0"),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// PHP `empty($value[$key])`.
pub fn empty(value: Option<&Value>) -> bool {
    value.map(|v| !truthy(v)).unwrap_or(true)
}

/// PHP `(string) $value`.
pub fn strval(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(true) => "1".to_string(),
        Value::Bool(false) => String::new(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else {
                float_to_string(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => "Array".to_string(),
    }
}

/// `(string) ($value ?? $default)`.
pub fn strval_or(value: Option<&Value>, default: &str) -> String {
    value.map(strval).unwrap_or_else(|| default.to_string())
}

/// PHP `(int) $value`.
pub fn intval(value: &Value) -> i64 {
    match value {
        Value::Null => 0,
        Value::Bool(b) => i64::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(u) = n.as_u64() {
                i64::try_from(u).unwrap_or(i64::MAX)
            } else {
                float_to_int(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => string_to_int(s),
        Value::Array(a) => i64::from(!a.is_empty()),
        Value::Object(o) => i64::from(!o.is_empty()),
    }
}

/// `(int) ($value ?? $default)`.
pub fn intval_or(value: Option<&Value>, default: i64) -> i64 {
    value.map(intval).unwrap_or(default)
}

/// PHP `(float) $value`.
pub fn floatval(value: &Value) -> f64 {
    match value {
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => numeric_prefix(s).map(|(f, _)| f).unwrap_or(0.0),
        Value::Array(a) => f64::from(u8::from(!a.is_empty())),
        Value::Object(o) => f64::from(u8::from(!o.is_empty())),
    }
}

fn float_to_int(f: f64) -> i64 {
    if !f.is_finite() {
        return 0;
    }
    // PHP 8 on 64-bit: out-of-range floats are undefined; saturate instead.
    f.trunc() as i64
}

fn is_php_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Parses the leading numeric part of a string the way PHP's numeric-string
/// rules do. Returns the value and whether the *whole* string was numeric.
fn numeric_prefix(s: &str) -> Option<(f64, bool)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && is_php_space(bytes[i]) {
        i += 1;
    }
    let start = i;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - int_start;
    if i < bytes.len() && bytes[i] == b'.' {
        let frac_start = i + 1;
        let mut j = frac_start;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if digits > 0 || j > frac_start {
            digits += j - frac_start;
            i = j;
        }
    }
    if digits == 0 {
        return None;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    let number: f64 = s[start..i].parse().ok()?;
    let mut k = i;
    while k < bytes.len() && is_php_space(bytes[k]) {
        k += 1;
    }
    Some((number, k == bytes.len()))
}

fn string_to_int(s: &str) -> i64 {
    let trimmed = s.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let integer_only = i == bytes.len() || !matches!(bytes[i], b'.' | b'e' | b'E');
    if integer_only {
        if i == digits_start {
            return 0;
        }
        return trimmed[..i].parse::<i64>().unwrap_or_else(|_| {
            if bytes[0] == b'-' {
                i64::MIN
            } else {
                i64::MAX
            }
        });
    }
    numeric_prefix(s).map(|(f, _)| float_to_int(f)).unwrap_or(0)
}

/// PHP `is_numeric()`.
pub fn is_numeric(value: &Value) -> bool {
    match value {
        Value::Number(_) => true,
        Value::String(s) => matches!(numeric_prefix(s), Some((_, true))),
        _ => false,
    }
}

/// PHP 8 `(string) $float` (uses the `precision` ini setting, 14 digits).
pub fn float_to_string(f: f64) -> String {
    if f.is_nan() {
        return "NAN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "INF" } else { "-INF" }.to_string();
    }
    if f == 0.0 {
        return if f.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    const PRECISION: usize = 14;
    // Round to 14 significant digits.
    let sci = format!("{:.*e}", PRECISION - 1, f.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if f < 0.0 { "-" } else { "" };
    let decpt = exp + 1; // position of the decimal point
    if decpt < -3 || decpt > PRECISION as i32 {
        let (first, rest) = digits.split_at(1);
        let rest = if rest.is_empty() { "0" } else { rest };
        let exp_sign = if exp < 0 { '-' } else { '+' };
        return format!("{sign}{first}.{rest}E{exp_sign}{}", exp.abs());
    }
    if decpt <= 0 {
        let zeros = "0".repeat((-decpt) as usize);
        return format!("{sign}0.{zeros}{digits}");
    }
    let decpt = decpt as usize;
    if digits.len() <= decpt {
        let zeros = "0".repeat(decpt - digits.len());
        format!("{sign}{digits}{zeros}")
    } else {
        format!("{sign}{}.{}", &digits[..decpt], &digits[decpt..])
    }
}

/// PHP `number_format($value, $decimals, '.', '')`, including PHP's
/// round-half-away-from-zero on the value as written (15 significant digits).
pub fn number_format(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return float_to_string(value);
    }
    let negative = value < 0.0;
    let sci = format!("{:.14e}", value.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i64 = exp.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa
        .bytes()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c - b'0')
        .collect();
    // digits[0] has weight 10^exp. Build integer part / fraction part.
    // Number of digits that belong in the result: (exp + 1) integer digits + decimals.
    let keep = exp + 1 + decimals as i64;
    let mut int_and_frac: Vec<u8>;
    if keep < 0 {
        int_and_frac = Vec::new();
    } else {
        let keep = keep as usize;
        let round_up = digits.get(keep).map(|d| *d >= 5).unwrap_or(false);
        digits.resize(keep.max(digits.len()), 0);
        int_and_frac = digits[..keep].to_vec();
        if round_up {
            let mut i = int_and_frac.len();
            let mut carry = true;
            while carry && i > 0 {
                i -= 1;
                if int_and_frac[i] == 9 {
                    int_and_frac[i] = 0;
                } else {
                    int_and_frac[i] += 1;
                    carry = false;
                }
            }
            if carry {
                int_and_frac.insert(0, 1);
            }
        }
    }
    // Left-pad so there is at least one integer digit.
    let total_needed = decimals + 1;
    while int_and_frac.len() < total_needed {
        int_and_frac.insert(0, 0);
    }
    let split = int_and_frac.len() - decimals;
    let int_part: String = int_and_frac[..split]
        .iter()
        .map(|d| (b'0' + d) as char)
        .collect();
    let int_part = int_part.trim_start_matches('0');
    let int_part = if int_part.is_empty() { "0" } else { int_part };
    let frac_part: String = int_and_frac[split..]
        .iter()
        .map(|d| (b'0' + d) as char)
        .collect();
    let is_zero = int_part == "0" && frac_part.bytes().all(|b| b == b'0');
    let sign = if negative && !is_zero { "-" } else { "" };
    if decimals == 0 {
        format!("{sign}{int_part}")
    } else {
        format!("{sign}{int_part}.{frac_part}")
    }
}

/// PHP `trim()` with the default character list.
pub fn trim(s: &str) -> &str {
    s.trim_matches([' ', '\t', '\n', '\r', '\0', '\x0b'])
}

/// PHP `strtolower()` (PHP 8: ASCII only).
pub fn lower(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// PHP `strtoupper()` (PHP 8: ASCII only).
pub fn upper(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// PHP `ucfirst()`.
pub fn ucfirst(s: &str) -> String {
    let mut out = s.to_string();
    if let Some(first) = out.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    out
}

/// PHP `rtrim($s, $chars)` for a set of single-byte characters.
pub fn rtrim_chars<'a>(s: &'a str, chars: &[char]) -> &'a str {
    s.trim_end_matches(chars)
}

/// PHP `strcasecmp($a, $b) === 0` (ASCII case-insensitive equality).
pub fn strcaseeq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// PHP `urlencode()`.
pub fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// PHP `urldecode()` (used by tests to read query strings back).
pub fn urldecode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
            }
            other => out.push(other),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// PHP `http_build_query($data)` with its default settings: `null` values are
/// skipped, `true`/`false` become `1`/`0`, nested arrays become `key[sub]=`.
pub fn http_build_query(data: &Value) -> String {
    let mut parts = Vec::new();
    match data {
        Value::Object(map) => {
            for (k, v) in map {
                build_query_part(&urlencode(k), v, &mut parts);
            }
        }
        Value::Array(list) => {
            for (i, v) in list.iter().enumerate() {
                build_query_part(&i.to_string(), v, &mut parts);
            }
        }
        _ => {}
    }
    parts.join("&")
}

fn build_query_part(key: &str, value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::Null => {}
        Value::Bool(b) => parts.push(format!("{key}={}", u8::from(*b))),
        Value::Number(_) | Value::String(_) => {
            parts.push(format!("{key}={}", urlencode(&strval(value))))
        }
        Value::Object(map) => {
            for (k, v) in map {
                build_query_part(&format!("{key}%5B{}%5D", urlencode(k)), v, parts);
            }
        }
        Value::Array(list) => {
            for (i, v) in list.iter().enumerate() {
                build_query_part(&format!("{key}%5B{i}%5D"), v, parts);
            }
        }
    }
}

/// Builds a JSON object from `(key, value)` pairs, keeping their order.
pub fn obj<const N: usize>(pairs: [(&str, Value); N]) -> Value {
    let mut map = Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

/// An empty PHP array encodes as `[]` even where an object is meant; this
/// keeps that shape so outputs match php-dna byte for byte.
pub fn empty_array() -> Value {
    Value::Array(Vec::new())
}

/// A PHP associative array that may be empty: `[]` when empty, else an object.
pub fn assoc_or_empty(map: Map<String, Value>) -> Value {
    if map.is_empty() {
        empty_array()
    } else {
        Value::Object(map)
    }
}

/// PHP `(array) $value`.
pub fn to_array(value: &Value) -> Value {
    match value {
        Value::Null => empty_array(),
        Value::Array(_) | Value::Object(_) => value.clone(),
        other => Value::Array(vec![other.clone()]),
    }
}

/// Iterate the values of a PHP array (list items or object values).
pub fn values(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(list) => list.iter().collect(),
        Value::Object(map) => map.values().collect(),
        _ => Vec::new(),
    }
}

/// Iterate `(key, value)` pairs of a PHP array (list indexes become keys).
pub fn entries(value: &Value) -> Vec<(String, &Value)> {
    match value {
        Value::Array(list) => list
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v))
            .collect(),
        Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v)).collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------

/// Days since 1970-01-01 for a proleptic Gregorian date (month/day may
/// overflow, as with PHP's date parser: 2023-02-31 is 2023-03-03).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    // Normalise month into 1..=12 adjusting the year.
    let mut y = year + (month - 1).div_euclid(12);
    let m = (month - 1).rem_euclid(12) + 1;
    if m <= 2 {
        y -= 1;
    }
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + 1 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468 + (day - 1)
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn take_digits(bytes: &[u8], pos: &mut usize, min: usize, max: usize) -> Option<i64> {
    let start = *pos;
    while *pos < bytes.len() && *pos - start < max && bytes[*pos].is_ascii_digit() {
        *pos += 1;
    }
    if *pos - start < min {
        return None;
    }
    std::str::from_utf8(&bytes[start..*pos]).ok()?.parse().ok()
}

/// A subset of PHP `strtotime()` covering what the gateway sends: ISO 8601
/// dates `YYYY-MM-DD`, optionally followed by `T` or a space and
/// `HH:MM[:SS[.fraction]]`, and an optional `Z` / `±HH:MM` / `±HHMM` offset.
/// Times without an offset are read as UTC. Returns `None` when PHP's
/// `strtotime()` would return `false` for these shapes.
pub fn strtotime(input: &str) -> Option<i64> {
    let s = input.trim();
    let b = s.as_bytes();
    let mut p = 0;
    let year = take_digits(b, &mut p, 4, 4)?;
    if b.get(p) != Some(&b'-') {
        return None;
    }
    p += 1;
    let month = take_digits(b, &mut p, 1, 2)?;
    if b.get(p) != Some(&b'-') {
        return None;
    }
    p += 1;
    let day = take_digits(b, &mut p, 1, 2)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (mut hour, mut minute, mut second) = (0, 0, 0);
    let mut offset = 0i64;
    if p < b.len() && (b[p] == b'T' || b[p] == b't' || b[p] == b' ') {
        p += 1;
        hour = take_digits(b, &mut p, 1, 2)?;
        if b.get(p) != Some(&b':') {
            return None;
        }
        p += 1;
        minute = take_digits(b, &mut p, 2, 2)?;
        if b.get(p) == Some(&b':') {
            p += 1;
            second = take_digits(b, &mut p, 2, 2)?;
            if b.get(p) == Some(&b'.') || b.get(p) == Some(&b',') {
                p += 1;
                take_digits(b, &mut p, 1, 9)?;
            }
        }
        if hour > 24 || minute > 59 || second > 60 {
            return None;
        }
    }
    if p < b.len() {
        match b[p] {
            b'Z' | b'z' => p += 1,
            b'+' | b'-' => {
                let sign = if b[p] == b'-' { -1 } else { 1 };
                p += 1;
                let oh = take_digits(b, &mut p, 2, 2)?;
                if b.get(p) == Some(&b':') {
                    p += 1;
                }
                let om = take_digits(b, &mut p, 0, 2).unwrap_or(0);
                offset = sign * (oh * 3600 + om * 60);
            }
            _ => return None,
        }
    }
    if p != b.len() {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3600 + minute * 60 + second - offset)
}

/// PHP `date('Y-m-d\TH:i:s', $ts)` in UTC.
pub fn format_date(ts: i64) -> String {
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let year = if y < 0 {
        format!("-{:04}", -y)
    } else {
        format!("{y:04}")
    };
    format!(
        "{year}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Current Unix time in seconds.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Current Unix time in fractional seconds.
pub fn microtime() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// PHP `json_encode($value)` (compact; slashes and Unicode are not escaped,
/// which only changes the wire encoding, not the decoded value).
pub fn json_encode(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn float_strings_match_php() {
        assert_eq!(float_to_string(601.64), "601.64");
        assert_eq!(float_to_string(1250.5), "1250.5");
        assert_eq!(float_to_string(99.999), "99.999");
        assert_eq!(float_to_string(1.0), "1");
        assert_eq!(float_to_string(0.1 + 0.2), "0.3");
        assert_eq!(float_to_string(1e14), "1.0E+14");
        assert_eq!(float_to_string(12345678901234.0), "12345678901234");
        assert_eq!(float_to_string(0.0001), "0.0001");
        assert_eq!(float_to_string(0.00001), "1.0E-5");
        assert_eq!(float_to_string(-2.5), "-2.5");
    }

    #[test]
    fn number_format_matches_php() {
        assert_eq!(number_format(11.99, 4), "11.9900");
        assert_eq!(number_format(2500.0, 4), "2500.0000");
        assert_eq!(number_format(601.64, 2), "601.64");
        assert_eq!(number_format(1.005, 2), "1.01");
        assert_eq!(number_format(0.125, 2), "0.13");
        assert_eq!(number_format(0.0, 2), "0.00");
        assert_eq!(number_format(99.999, 2), "100.00");
        assert_eq!(number_format(-1.5, 0), "-2");
        assert_eq!(number_format(-0.001, 2), "0.00");
        assert_eq!(number_format(0.0004, 4), "0.0004");
    }

    #[test]
    fn casts_match_php() {
        assert_eq!(intval(&json!("12abc")), 12);
        assert_eq!(intval(&json!("1e3")), 1000);
        assert_eq!(intval(&json!("00000000-0000")), 0);
        assert_eq!(intval(&json!(3.9)), 3);
        assert_eq!(intval(&json!(true)), 1);
        assert_eq!(strval(&json!(true)), "1");
        assert_eq!(strval(&json!(false)), "");
        assert!(is_numeric(&json!(" 12 ")));
        assert!(is_numeric(&json!("1.5e3")));
        assert!(!is_numeric(&json!("12a")));
        assert!(!is_numeric(&json!("")));
        assert!(truthy(&json!("0.0")));
        assert!(!truthy(&json!("0")));
    }

    #[test]
    fn query_building_matches_php() {
        let q = http_build_query(
            &json!({"a": "x y", "b": true, "c": false, "d": null, "e": [1, 2], "f": "ä/&"}),
        );
        assert_eq!(q, "a=x+y&b=1&c=0&e%5B0%5D=1&e%5B1%5D=2&f=%C3%A4%2F%26");
    }

    #[test]
    fn dates_parse_and_format() {
        assert_eq!(
            format_date(strtotime("2027-03-15T10:20:30").unwrap()),
            "2027-03-15T10:20:30"
        );
        assert_eq!(
            format_date(strtotime("0001-01-01T00:00:00").unwrap()),
            "0001-01-01T00:00:00"
        );
        assert_eq!(
            format_date(strtotime("2024-01-01T12:00:00+02:00").unwrap()),
            "2024-01-01T10:00:00"
        );
        assert_eq!(
            format_date(strtotime("2023-02-31").unwrap()),
            "2023-03-03T00:00:00"
        );
        assert_eq!(strtotime("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(strtotime("not-a-date"), None);
        assert_eq!(strtotime(""), None);
    }
}
