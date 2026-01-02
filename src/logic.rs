use iced_aw::core::date::Date;

pub fn iced_date_to_local_datetime(date: Date) -> Result<chrono::NaiveDate, String> {
    match chrono::NaiveDate::from_ymd_opt(date.year, date.month, date.day) {
        Some(n) => Ok(n),
        None => Err(String::from("Invalid date")),
    }

    // Switched this one to naive so no need for timezone accounting
    // let naive_dt: chrono::NaiveDateTime = match naive_date
    //     .and_hms_opt(0, 0, 0) {
    //         Some(t) => t,
    //         None => return Err(String::from("Invalid time"))
    //     };

    // match chrono::Local
    //     .from_local_datetime(&naive_dt)
    //     .single() {
    //         Some(l) => Ok(l),
    //         None => Err(String::from("ambiguous or invalid local datetime"))
    //     }
}
