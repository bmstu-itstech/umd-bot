use crate::domain::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Service {
    InitialRegistration,
    Visa,
    RenewalOfRegistration,
    RenewalOfVisa,
    All,
    Consultation,
}

impl Service {
    pub fn has_deadline(&self) -> bool {
        matches!(self, Self::InitialRegistration | Self::Visa | Self::All)
    }

    pub fn all() -> &'static [Service] {
        &[
            Service::InitialRegistration,
            Service::Visa,
            Service::RenewalOfRegistration,
            Service::RenewalOfVisa,
            Service::All,
            Service::Consultation,
        ]
    }
}

impl From<Service> for String {
    fn from(val: Service) -> String {
        match val {
            Service::InitialRegistration => "initial_registration".into(),
            Service::Visa => "visa".into(),
            Service::RenewalOfRegistration => "renewal_of_registration".into(),
            Service::RenewalOfVisa => "renewal_of_visa".into(),
            Service::All => "all".into(),
            Service::Consultation => "consultation".into(),
        }
    }
}

impl TryFrom<String> for Service {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "initial_registration" => Ok(Self::InitialRegistration),
            "visa" => Ok(Self::Visa),
            "renewal_of_registration" => Ok(Self::RenewalOfRegistration),
            "renewal_of_visa" => Ok(Self::RenewalOfVisa),
            "all" => Ok(Self::All),
            "consultation" => Ok(Self::Consultation),
            _ => Err(Error::InvalidValue(format!(
                "invalid Service: expected one of ['initial_registration', 'visa', 'insurance', \
                'visa_and_insurance', 'renewal_of_registration', 'renewal_of_visa', 'all'], got {}",
                value
            ))),
        }
    }
}
