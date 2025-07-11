use chrono::{NaiveDate, NaiveTime, Utc};

use crate::domain::Error;
use crate::domain::models::Interval;


#[derive(Debug, Clone)]
pub struct NaiveTimeInterval {
    start: NaiveTime,
    end:   NaiveTime,
}

impl NaiveTimeInterval {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Result<Self, Error> {
        if start > end {
            return Err(Error::InvalidInterval(
                format!("expected start < end, got {:?} -- {:?}", start, end)
            ))
        }

        if start == end {
            return Err(Error::InvalidInterval(
                format!("expected not zero interval, got {:?} -- {:?}", start, end)
            ))
        }

        Ok(Self { start, end })
    }
    
    pub fn and_date(self, date: NaiveDate) -> Interval<Utc> {
        Interval::new(
            date.and_time(self.start).and_utc(),
            date.and_time(self.end).and_utc(),
        )
            .unwrap()   // Гарантируется порядок start < end и нахождение в пределах одного дня
    }
    
    pub fn start(&self) -> &NaiveTime {
        &self.start
    }
    
    pub fn end(&self) -> &NaiveTime {
        &self.end
    }
}
