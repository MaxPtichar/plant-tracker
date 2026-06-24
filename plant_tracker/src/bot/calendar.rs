use chrono::{Datelike, Days, Months, NaiveDate};

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

const IGNORE: &str = "ignore";

pub fn build_calendar(start_date: NaiveDate) -> InlineKeyboardMarkup {
    let days = Days::new(start_date.day() as u64 - 1); // tests

    let days_start = start_date.checked_sub_days(days).unwrap().day(); // tests

    let month_start = start_date.month();

    let years_start = start_date.year();

    let start_month = NaiveDate::from_ymd_opt(years_start, month_start, days_start).unwrap();

    let month = Months::new(1);

    let next_date = start_month.checked_add_months(month).unwrap();
    let end_date = next_date;

    dbg!(&end_date);

    let dates = start_month
        .iter_days()
        .take_while(|&date| date != end_date)
        .collect::<Vec<NaiveDate>>();

    calendar_button(dates)
}

fn calendar_grid(dates: &[NaiveDate]) -> [Option<NaiveDate>; 42] {
    let start_date = dates.first().unwrap();
    let day_offset: usize = start_date.weekday().number_from_monday() as usize - 1;

    let mut grid: [Option<NaiveDate>; 42] = [None; 42];

    for i in 0..dates.len() {
        grid[i + day_offset] = Some(dates[i]);
    }

    grid
}

pub fn year_grid(date: &NaiveDate) -> InlineKeyboardMarkup {
    let current_year = date.year();

    let start_date = current_year - 4;
    let end_date = current_year + 4;

    let prev_years = current_year - 9;
    let next_years: i32 = current_year + 9;

    let mut grid: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let switcher = vec![
        InlineKeyboardButton::callback("<<", format!("call:navyear:01.01.{}", prev_years)),
        InlineKeyboardButton::callback(">>", format!("call:navyear:01.01.{}", next_years)),
    ];

    let year_butt = (start_date..=end_date)
        .into_iter()
        .map(|year| {
            InlineKeyboardButton::callback(year.to_string(), format!("call:navmonth:01.01.{year}",))
        })
        .collect::<Vec<InlineKeyboardButton>>()
        .chunks(3)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<InlineKeyboardButton>>>();

    grid.extend(year_butt);
    grid.push(switcher);

    InlineKeyboardMarkup::new(grid)
}

pub fn month_grid(date: &NaiveDate) -> InlineKeyboardMarkup {
    let year = date.year();

    let month_names = [
        "Янв", "Фев", "Март", "Апр", "Май", "Июнь", "Июль", "Авг", "Сент", "Окт", "Нояб", "Дек",
    ];

    let month_button = month_names
        .iter()
        .enumerate()
        .map(|(i, m)| {
            InlineKeyboardButton::callback(
                m.to_string(),
                format!("call:calendar:01.{}.{}", i + 1, year),
            )
        })
        .collect::<Vec<InlineKeyboardButton>>()
        .chunks(3)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<InlineKeyboardButton>>>();

    InlineKeyboardMarkup::new(month_button)
}

pub fn calendar_button(dates: Vec<NaiveDate>) -> InlineKeyboardMarkup {
    let start_date = dates.first().unwrap();

    let grid = calendar_grid(&dates);

    let year_button = year_button(start_date);

    let prev_next_month = prev_next_month(start_date);

    let mut calendar: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    calendar.push(year_button);
    calendar.push(prev_next_month);

    calendar.push(week_days());

    let month: Vec<Vec<InlineKeyboardButton>> = grid
        .iter()
        .filter_map(|opt_date| {
            if let Some(date) = opt_date {
                let button = InlineKeyboardButton::callback(
                    date.day().to_string(),
                    format!("{}.{}.{}", date.day(), date.month(), date.year()),
                );

                Some(button)
            } else {
                let button = InlineKeyboardButton::callback(" ", IGNORE);

                Some(button)
            }
        })
        .collect::<Vec<InlineKeyboardButton>>()
        .chunks(7)
        .enumerate()
        .map(|(row, chunk)| (row, chunk))
        .take_while(|(row, _)| *row != 6)
        .map(|(_, chunk)| chunk.to_vec())
        .collect();

    calendar.extend(month);

    let res = InlineKeyboardMarkup::new(calendar);

    res
}

fn week_days() -> Vec<InlineKeyboardButton> {
    let days_name: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

    days_name
        .iter()
        .map(|day| InlineKeyboardButton::callback(*day, IGNORE))
        .collect::<Vec<InlineKeyboardButton>>()
}

pub fn prev_next_month(date: &NaiveDate) -> Vec<InlineKeyboardButton> {
    let prev_month = date.checked_sub_months(Months::new(1)).unwrap();
    let next_month = date.checked_add_months(Months::new(1)).unwrap();

    let display_month = to_text_month(date);

    vec![
        InlineKeyboardButton::callback("<<", format!("move_to_{}", prev_month.format("%d.%m.%Y"))),
        InlineKeyboardButton::callback(
            format!("{display_month}"),
            format!("call:navmonth:{}", date.format("%d.%m.%Y")),
        ),
        InlineKeyboardButton::callback(">>", format!("move_to_{}", next_month.format("%d.%m.%Y"))),
    ]
}

/// return tuple (month, year)
pub fn year_button(date: &NaiveDate) -> Vec<InlineKeyboardButton> {
    dbg!(date.year().to_string());

    vec![InlineKeyboardButton::callback(
        date.year().to_string(),
        format!("call:navyear:01.01.{}", date.year().to_string()),
    )]
}

fn to_text_month(date: &NaiveDate) -> String {
    let month = date.month();

    let month_names = [
        "offset",
        "Январь",
        "Февраль",
        "Март",
        "Апрель",
        "Май",
        "Июнь",
        "Июль",
        "Август",
        "Сентябрь",
        "Октябрь",
        "Ноябрь",
        "Декабрь",
    ];

    format!("{}", month_names[month as usize])
}
