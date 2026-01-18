use crate::ast::*;
use crate::domain::*;
use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc, Weekday};
use std::str::FromStr;

pub struct Scheduler;

impl Scheduler {
    pub fn next_occurrence(
        def: &ValueDef,
        now: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        // Find Timeframe property
        let mut timeframe_groups = None;
        for prop in &def.properties {
            if let Property::Timeframe(groups) = prop {
                timeframe_groups = Some(groups);
                break;
            }
        }

        let groups = timeframe_groups?;
        let mut next_times = Vec::new();

        for group in groups {
            if let Some(next) = Self::next_for_group(group, now) {
                next_times.push(next);
            }
        }

        next_times.into_iter().min()
    }

    fn next_for_group(selectors: &Vec<RangeSelector>, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Separate selectors into Day constraints and Time constraints
        // Assumption: One day range and one time range per group (as seen in copd plan)
        // If multiple, intersection logic is needed.
        
        // Simplified Logic:
        // 1. Identify Day range (e.g. Mon-Fri)
        // 2. Identify Time range (e.g. 07:40-07:50)
        // 3. Iterate starting from 'now', checking if day matches and if time is future (for today) or simple start time (for future days).
        
        let mut start_day = None;
        let mut end_day = None;
        let mut start_time = None;
        // let mut end_time = None; // We only care about start time for triggering "begin of"

        for sel in selectors {
            match sel {
                RangeSelector::Between(e1, e2) | RangeSelector::Range(e1, e2) => {
                    // Check if it's Day range
                    if let (Some(d1), Some(d2)) = (Self::eval_weekday(e1), Self::eval_weekday(e2)) {
                        start_day = Some(d1);
                        end_day = Some(d2);
                    }
                    // Check if it's Time range
                    else if let (Some(t1), Some(_t2)) = (Self::eval_time(e1), Self::eval_time(e2)) {
                        start_time = Some(t1);
                        // end_time = Some(t2);
                    }
                }
                _ => {}
            }
        }

        let start_time = start_time.unwrap_or(NaiveTime::from_hms(0, 0, 0)); // Default midnight?
        
        // Search next 14 days
        for i in 0..14 {
            let candidate_date = now.date_naive() + chrono::Duration::days(i);
            let candidate_weekday = candidate_date.weekday();
            
            // Check day constraint
            let day_match = if let (Some(s), Some(e)) = (start_day, end_day) {
                Self::weekday_in_range(candidate_weekday, s, e)
            } else {
                true // No day constraint
            };

            if day_match {
                // Construct candidate datetime
                // If i==0 (today), time must be > now (or >= depending on exact semantics)
                // Actually, if we are simulating sequential execution, strict > might be needed if we just executed previous.
                // But for "Begin of period", it triggers exactly at start_time.
                // If now > start_time, we missed it for today.
                
                // Construct full DateTime
                // Note: creating DateTime from naive needs validation (DST etc)
                // Using simplistic approach
                let candidate_dt_naive = candidate_date.and_time(start_time);
                let candidate_dt = DateTime::<Utc>::from_utc(candidate_dt_naive, Utc); // Assuming simple mapping

                if candidate_dt > now {
                    return Some(candidate_dt);
                }
            }
        }
        
        None
    }

    fn eval_weekday(expr: &Expression) -> Option<Weekday> {
        match expr {
            Expression::Variable(s) => match s.to_lowercase().as_str() {
                "monday" => Some(Weekday::Mon),
                "tuesday" => Some(Weekday::Tue),
                "wednesday" => Some(Weekday::Wed),
                "thursday" => Some(Weekday::Thu),
                "friday" => Some(Weekday::Fri),
                "saturday" => Some(Weekday::Sat),
                "sunday" => Some(Weekday::Sun),
                _ => None,
            },
            _ => None,
        }
    }

    fn eval_time(expr: &Expression) -> Option<NaiveTime> {
        match expr {
            Expression::Literal(Literal::TimeOfDay(s)) => {
                NaiveTime::parse_from_str(s, "%H:%M").ok()
            },
            Expression::Literal(Literal::String(s)) => {
                 NaiveTime::parse_from_str(s, "%H:%M").ok()
            }
            _ => None,
        }
    }

    fn weekday_in_range(target: Weekday, start: Weekday, end: Weekday) -> bool {
        let t_idx = target.num_days_from_monday();
        let s_idx = start.num_days_from_monday();
        let e_idx = end.num_days_from_monday();
        
        if s_idx <= e_idx {
            t_idx >= s_idx && t_idx <= e_idx
        } else {
            // Wrap around (e.g. Fri ... Mon)
            t_idx >= s_idx || t_idx <= e_idx
        }
    }
}
