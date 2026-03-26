// this module has datetime stuff for gtfs for raptor and recalculating stop times

const MIDNIGHT_SECONDS: i32 = 24 * 60 * 60;

pub type Seconds = usize;

pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    weekday: DayOfWeek,
}

// because GTFS times allows times past midnight to be counted as part of same day
pub enum Day {
    CurrentDay,
    NextDay,
}

pub struct Time {
    pub day: Day,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
}

pub struct DateTime {
    date: Date,
    time: Time,
}

impl Time {
    pub fn new(hour: i32, minute: i32, second: i32) -> Self {
        let day: Day;
        if hour >= 24 {
            day = Day::NextDay;
        } else {
            day = Day::CurrentDay;
        }

        Self {
            day,
            hour,
            minute,
            second,
        }
    }
}
