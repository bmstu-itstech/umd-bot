use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub struct ClosedRange<T>
where
    T: Clone + PartialOrd + PartialEq,
{
    pub start: T,
    pub end: T,
}

pub struct ClosedRangeIter<T>
where
    T: PartialOrd,
{
    current: Option<T>,
    end: T,
}

impl<T> ClosedRange<T>
where
    T: Clone + PartialOrd + PartialEq,
{
    pub fn iter(&self) -> ClosedRangeIter<T>
    where
        T: PartialOrd + Clone,
    {
        ClosedRangeIter {
            current: Some(self.start.clone()),
            end: self.end.clone(),
        }
    }

    pub fn into_iter(self) -> ClosedRangeIter<T>
    where
        T: PartialOrd,
    {
        ClosedRangeIter {
            current: Some(self.start),
            end: self.end,
        }
    }

    pub fn contains(&self, other: &ClosedRange<T>) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn overlaps(&self, other: &ClosedRange<T>) -> bool {
        self.start < other.end && self.end > other.start
    }
}

impl Iterator for ClosedRangeIter<NaiveDate> {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        if current < self.end {
            self.current = current.succ_opt();
            Some(current)
        } else {
            None
        }
    }
}
