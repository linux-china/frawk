use std::time::SystemTime;
use chrono::{Datelike, DateTime, Local, NaiveDateTime, Timelike, TimeZone};
use crate::runtime;
use crate::runtime::{Int, Str};

const WEEKS: [&'static str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

pub fn strftime(format: &str, timestamp: i64) -> String {
    let utc_now = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let local_now: DateTime<Local> = Local.from_utc_datetime(&utc_now);
    local_now.format(&format.to_string()).to_string()
}

pub fn mktime(date_time_text: &str, timezone: i64) -> u64 {
    let dt_text_timezone = if timezone > 0 {
        format!("{} {}", date_time_text, timezone_offset_text(timezone))
    } else {
        date_time_text.to_string()
    };
    if let Ok(date_time) = dateparser::parse(&dt_text_timezone) {
        return date_time.timestamp() as u64;
    } else {
        // fend date format: Thursday, 20 May 2021
        if is_fend_date(&dt_text_timezone) {
            let adjusted_dt_text = &dt_text_timezone[dt_text_timezone.find(' ').unwrap() + 1..];
            if let Ok(date_time) = dateparser::parse(&adjusted_dt_text) {
                return date_time.timestamp() as u64;
            }
        }
        let dt_text = format!("{} {}", date_time_text, timezone_offset_text(timezone));
        //gawk compatible parser
        if let Ok(date_time) = DateTime::parse_from_str(&dt_text, "%Y %m %d %H %M %S %z") {
            return date_time.timestamp() as u64;
        }
    }
    0
}

fn is_fend_date(text: &str) -> bool {
    if text.contains(',') {
        let temp = &text[0..text.find(',').unwrap()];
        return WEEKS.contains(&temp);
    }
    false
}

fn timezone_offset_text(timezone: i64) -> String {
    if timezone >= 10 {
        format!("+{}:00", timezone)
    } else if timezone >= 0 && timezone < 10 {
        format!("+0{}:00", timezone)
    } else {
        "+00:00".to_owned()
    }
}

pub(crate) fn datetime<'a>(date_time_text: &str) -> runtime::StrMap<'a, Int> {
    if date_time_text.is_empty() {
        let seconds = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
        return datetime2(seconds);
    } else if let Ok(timestamp) = date_time_text.parse::<i64>() {
        datetime2(timestamp)
    } else {
        let timestamp = mktime(date_time_text, 0);
        datetime2(timestamp as i64)
    }
}

pub(crate) fn datetime2<'a>(timestamp: i64) -> runtime::StrMap<'a, Int> {
    let result: runtime::StrMap<Int> = runtime::StrMap::default();
    let utc_now = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    result.insert(Str::from("second"), utc_now.second() as Int);
    result.insert(Str::from("minute"), utc_now.minute() as Int);
    result.insert(Str::from("hour"), utc_now.hour() as Int);
    result.insert(Str::from("althour"), utc_now.hour12().1 as Int);
    result.insert(Str::from("monthday"), utc_now.day() as Int);
    result.insert(Str::from("month"), utc_now.month() as Int);
    result.insert(Str::from("year"), utc_now.year() as Int);
    result.insert(Str::from("weekday"), utc_now.weekday() as Int);
    result.insert(Str::from("yearday"), utc_now.ordinal() as Int);
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_parse() {
        let date_text_items = vec!["Thursday, 20 May 2021"];
        for item in date_text_items {
            mktime(item, 0);
        }
    }

    #[test]
    fn test_fend_date() {
        let text = "Thursday, 20 May 2021";
        println!("{}", is_fend_date(text));
    }

    #[test]
    fn test_datetime() {
        let result = datetime("1575043680");
        println!("{:?}", result);
    }
}