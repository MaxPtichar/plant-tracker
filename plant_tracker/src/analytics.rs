use crate::models::PlantWithLastFeedWatering;

/// Returns days elapsed since the last feed-watering measurement.
///
/// # Returns
/// - `Some(days)` — days since last `AfterWateringWithFeed`
/// - `None` — no feed-watering recorded yet
pub fn days_from_last_feed(last_feed: &PlantWithLastFeedWatering) -> Option<u32> {
    let date = last_feed.date?;
    let days = (chrono::Local::now().date_naive() - date).num_days();

    Some(days.max(0) as u32)
}
