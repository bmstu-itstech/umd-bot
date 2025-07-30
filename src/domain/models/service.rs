use crate::domain::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Service {
    InitialRegistration,
    Visa,
    RenewalOfRegistration,
    RenewalOfVisa,
    All,
}

impl Service {
    pub fn has_deadline(&self) -> bool {
        match self {
            Self::InitialRegistration
            | Self::Visa
            | Self::All => true,
            _ => false,
        }
    }

    pub fn all() -> &'static [Service] {
        &[
            Service::InitialRegistration,
            Service::Visa,
            Service::RenewalOfRegistration,
            Service::RenewalOfVisa,
            Service::All,
        ]
    }
}

impl Into<String> for Service {
    fn into(self) -> String {
        match self {
            Self::InitialRegistration => "initial_registration".into(),
            Self::Visa => "visa".into(),
            Self::RenewalOfRegistration => "renewal_of_registration".into(),
            Self::RenewalOfVisa => "renewal_of_visa".into(),
            Self::All => "all".into(),
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
            _ => Err(Error::InvalidValue(format!(
                "invalid Service: expected one of ['initial_registration', 'visa', 'insurance', \
                'visa_and_insurance', 'renewal_of_registration', 'renewal_of_visa', 'all'], got {}",
                value
            ))),
        }
    }
}
