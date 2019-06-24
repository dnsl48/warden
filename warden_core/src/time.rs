use yamlette::model::yaml::timestamp::TimestampValue;

pub fn yamlette_timestamp_value() -> TimestampValue {
    use chrono::{Datelike, Timelike};
    use yamlette::model::Fraction;
    use yamlette::model::yaml::float::FloatValue;

    let utc = chrono::offset::Utc::now().naive_utc();

    TimestampValue::new()
        .year(utc.year())
        .month(utc.month() as u8)
        .day(utc.day() as u8)
        .hour(utc.hour() as u8)
        .minute(utc.minute() as u8)
        .second(utc.second() as u8)
        .fraction(FloatValue::from(Fraction::new(utc.nanosecond() / 1000_000u32, 1000u32)))
        .tz_hour(0)
        .tz_minute(0)
}
