use spacetimedb::Timestamp;
use std::time::Duration;

const DAYS_PER_WEEK: u64 = 7;
const HOURS_PER_DAY: u64 = 24;
const MINS_PER_HOUR: u64 = 60;
const SECS_PER_MINUTE: u64 = 60;

pub trait DurationExt {
    fn from_weeks_ext(weeks: u64) -> Self;
    fn from_days_ext(days: u64) -> Self;
    fn from_hours_ext(hours: u64) -> Self;
    fn from_mins_ext(minutes: u64) -> Self;
}

impl DurationExt for Duration {
    fn from_weeks_ext(weeks: u64) -> Duration {
        let secs = weeks
            .saturating_mul(DAYS_PER_WEEK)
            .saturating_mul(HOURS_PER_DAY)
            .saturating_mul(MINS_PER_HOUR)
            .saturating_mul(SECS_PER_MINUTE);
        Duration::from_secs(secs)
    }

    fn from_days_ext(days: u64) -> Duration {
        let secs = days
            .saturating_mul(HOURS_PER_DAY)
            .saturating_mul(MINS_PER_HOUR)
            .saturating_mul(SECS_PER_MINUTE);
        Duration::from_secs(secs)
    }

    fn from_hours_ext(hours: u64) -> Duration {
        let secs = hours.saturating_mul(MINS_PER_HOUR).saturating_mul(SECS_PER_MINUTE);
        Duration::from_secs(secs)
    }

    fn from_mins_ext(minutes: u64) -> Duration {
        let secs = minutes.saturating_mul(SECS_PER_MINUTE);
        Duration::from_secs(secs)
    }
}

pub trait TimestampExt {
    fn into_midnight(self) -> Self;
}

impl TimestampExt for Timestamp {
    fn into_midnight(self) -> Self {
        let micros_per_sec = 1_000_000;
        let seconds_in_day = 24 * 60 * 60;
        let micros_since_epoch = self.to_micros_since_unix_epoch();
        let secs_since_epoch = micros_since_epoch / micros_per_sec;
        let secs_today = secs_since_epoch % seconds_in_day;
        let midnight_secs_since_epoch = secs_since_epoch - secs_today;
        Timestamp::from_micros_since_unix_epoch(midnight_secs_since_epoch * micros_per_sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb::Timestamp;
    use std::time::Duration;

    #[test]
    fn from_weeks_ext_converts_to_seconds() {
        assert_eq!(Duration::from_weeks_ext(1), Duration::from_secs(604_800));
    }

    #[test]
    fn from_mins_ext_converts_to_seconds() {
        assert_eq!(Duration::from_mins_ext(2), Duration::from_secs(120));
    }

    #[test]
    fn into_midnight_rounds_down_to_utc_midnight() {
        let current = Timestamp::from_micros_since_unix_epoch(1_745_748_000_000_000);
        let expected = Timestamp::from_micros_since_unix_epoch(1_745_712_000_000_000);
        assert_eq!(current.into_midnight(), expected);
    }
}
