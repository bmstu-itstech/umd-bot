use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Citizenship {
    Tajikistan,
    Uzbekistan,
    Kazakhstan,
    Kyrgyzstan,
    Armenia,
    Belarus,
    Ukraine,
    Other(String),
}

impl Citizenship {
    pub fn as_str(&self) -> &str {
        match self {
            Citizenship::Tajikistan => "Таджикистан",
            Citizenship::Uzbekistan => "Узбекистан",
            Citizenship::Kazakhstan => "Казахстан",
            Citizenship::Kyrgyzstan => "Кыргызстан",
            Citizenship::Armenia => "Армения",
            Citizenship::Belarus => "Беларусь",
            Citizenship::Ukraine => "Украина",
            Citizenship::Other(s) => s,
        }
    }
}

impl From<&str> for Citizenship {
    fn from(s: &str) -> Self {
        match s {
            "Таджикистан" => Citizenship::Tajikistan,
            "Узбекистан" => Citizenship::Uzbekistan,
            "Казахстан" => Citizenship::Kazakhstan,
            "Кыргызстан" => Citizenship::Kyrgyzstan,
            "Армения" => Citizenship::Armenia,
            "Беларусь" => Citizenship::Belarus,
            "Украина" => Citizenship::Ukraine,
            _ => Citizenship::Other(s.into()),
        }
    }
}

impl Into<String> for Citizenship {
    fn into(self) -> String {
        self.as_str().into()
    }
}
