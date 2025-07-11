use chrono::Days;

use crate::domain::models::Citizenship;


/// DeadlinePolicy описывает сроки подачи основных документов для иностранцев.
pub trait DeadlinePolicy {
    fn deadline(&self, citizenship: &Citizenship) -> Days;
}

/// StandardDeadlinePolicy устанавливает указанные представителем УМД сроки:
/// - 15 дней для граждан Таджикистана и Узбекистана;
/// - 30 дней для граждан Казахстана, Киргизстана и Армении;
/// - 90 дней для граждан Беларуси и Украины;
/// -  7 дней для граждан иных стран.
#[derive(Default)]
pub struct StandardDeadlinePolicy;

impl DeadlinePolicy for StandardDeadlinePolicy {
    fn deadline(&self, citizenship: &Citizenship) -> Days {
        match citizenship {
            Citizenship::Tajikistan
            | Citizenship::Uzbekistan => Days::new(15),

            Citizenship::Kazakhstan
            | Citizenship::Kyrgyzstan
            | Citizenship::Armenia => Days::new(30),

            Citizenship::Belarus
            | Citizenship::Ukraine => Days::new(90),

            Citizenship::Other(_) => Days::new(7),
        }
    }
}

#[cfg(test)]
mod default_deadline_policy_tests {
    use super::*;
    use crate::domain::services::Mon2FriWorkingHoursPolicy;
    
    fn setup_default_working_policy() -> Mon2FriWorkingHoursPolicy {
        Mon2FriWorkingHoursPolicy::default()
    }
 
    #[test]
    fn test_deadlines() {
        // GIVEN стандартная политика сроков
        let policy = StandardDeadlinePolicy::default();
        
        // (WHEN гражданство, THEN ожидаемое количество дней)
        let cases = vec![
            (Citizenship::Tajikistan, 15),
            (Citizenship::Uzbekistan, 15),
            (Citizenship::Kazakhstan, 30),
            (Citizenship::Kyrgyzstan, 30),
            (Citizenship::Armenia, 30),
            (Citizenship::Belarus, 90),
            (Citizenship::Ukraine, 90),
            (Citizenship::Other("China".to_string()), 7),
        ];
        
        cases.into_iter().for_each(|(citizenship, days)| {
            assert_eq!(policy.deadline(&citizenship), Days::new(days));
        })
    }
}
