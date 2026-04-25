// this module has datetime stuff for gtfs for raptor and recalculating stop times

use crate::utils::errors::TimeParsingError;

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

/// Takes a GTFS arrival or departure time in the format "HH:mm:ss" and returns the number of
/// seconds. May be greater than 60 * 60 * 24 because GTFS allows for after midnight times to be
/// like represented like so: 27:15:30
pub fn gtfs_time_to_seconds(gtfs_time: &str) -> Result<Seconds, TimeParsingError> {
    let time_components: Vec<&str> = gtfs_time.split(":").collect();

    if time_components.len() != 3 {
        return Err(TimeParsingError::InvalidFormat(gtfs_time.to_string()));
    }

    let hours = match time_components[0].parse::<usize>() {
        Ok(hour) => hour,
        Err(_) => {
            return Err(TimeParsingError::InvalidFormat(gtfs_time.to_string()));
        }
    };

    // minute and second obviously cannot be greater than 60
    let minutes = match time_components[1].parse::<usize>() {
        Ok(minute) if minute < 60 => minute,
        Ok(_) => {
            return Err(TimeParsingError::InvalidComponent(
                "minutes".to_string(),
                time_components[1].to_string(),
            ));
        }
        Err(_) => {
            return Err(TimeParsingError::InvalidComponent(
                "minutes".to_string(),
                time_components[1].to_string(),
            ));
        }
    };

    let seconds = match time_components[2].parse::<usize>() {
        Ok(second) if second < 60 => second,
        Ok(_) => {
            return Err(TimeParsingError::InvalidComponent(
                "seconds".to_string(),
                time_components[2].to_string(),
            ));
        }
        Err(_) => {
            return Err(TimeParsingError::InvalidComponent(
                "seconds".to_string(),
                time_components[2].to_string(),
            ));
        }
    };

    Ok(hours * 3600 + minutes * 60 + seconds)
}

pub fn seconds_to_gtfs_time(num_seconds: Seconds) -> String {
    let hours = num_seconds / 3600;
    let minutes = (num_seconds % 3600) / 60;
    let seconds = num_seconds % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
