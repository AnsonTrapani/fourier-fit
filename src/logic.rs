use iced_aw::core::date::Date;

pub fn iced_date_to_local_datetime(date: Date) -> Result<chrono::NaiveDate, String> {
    match chrono::NaiveDate::from_ymd_opt(date.year, date.month, date.day) {
        Some(n) => Ok(n),
        None => Err(String::from("Invalid date")),
    }
}
