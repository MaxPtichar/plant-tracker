use chrono::{NaiveDate, Utc};
use chrono_tz::Europe::Minsk;


#[allow(dead_code)]
pub fn get_minsk_date() -> NaiveDate {
    let minsk_time = Utc::now().with_timezone(&Minsk);

    minsk_time.date_naive()
}
