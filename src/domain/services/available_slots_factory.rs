use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::models::{book_all, Booking, Slot};
use crate::domain::services::{SlotsFactory, WorkingHoursPolicy};


pub trait AvailableSlotsFactory<const N: usize> {
    fn available_slots(&self, date: NaiveDate, bookings: &[Booking]) -> Result<Vec<Slot<N>>, Error>;
}

pub struct AvailableSlotsFactoryImpl<const N: usize, SF, WH>
where
    SF: SlotsFactory<N>,
    WH: WorkingHoursPolicy,
{
    slots_factory: SF,
    working_hours_policy: WH,
}

impl<const N: usize, SF, WH> AvailableSlotsFactory<N> for AvailableSlotsFactoryImpl<N, SF, WH>
where
    SF: SlotsFactory<N>,
    WH: WorkingHoursPolicy,
{
    fn available_slots(&self, date: NaiveDate, bookings: &[Booking]) -> Result<Vec<Slot<N>>, Error> {
        let bounds = match self.working_hours_policy.bounds(date) {
            Some(bounds) => bounds,
            None => return Ok(Vec::new()),
        };
        
        let mut slots: Vec<Slot<N>> = self.slots_factory
            .slots(&bounds)
            .collect();
        
        book_all(&mut slots, bookings)?;
        
        let available = slots
            .into_iter()
            .filter(|slot| slot.is_available())
            .collect();
        
        Ok(available)
    }
}
