#[derive(Debug, Clone)]
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

impl Into<String> for Citizenship {
    fn into(self) -> String {
        match self {
            Citizenship::Tajikistan => "Таджикистан".to_string(),
            Citizenship::Uzbekistan => "Узбекистан".to_string(),
            Citizenship::Kazakhstan => "Казахстан".to_string(),
            Citizenship::Kyrgyzstan => "Кыргызстан".to_string(),
            Citizenship::Armenia    => "Армения".to_string(),
            Citizenship::Belarus    => "Беларусь".to_string(),
            Citizenship::Ukraine    => "Украина".to_string(),
            Citizenship::Other(s) => s,
        }
    }
}

impl From<String> for Citizenship {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Таджикистан" => Citizenship::Tajikistan,
            "Узбекистан"  => Citizenship::Uzbekistan,
            "Казахстан"   => Citizenship::Kazakhstan,
            "Кыргызстан"  => Citizenship::Kyrgyzstan,
            "Армения"     => Citizenship::Armenia,
            "Беларусь"    => Citizenship::Belarus,
            "Украина"     => Citizenship::Ukraine,
            s => Citizenship::Other(s.into()),
        }
    }
}
