use crate::domain::models::Service;
use crate::usecases::FreeSlotDTO;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub const AGREEMENT_BTN: &'static str = "Подтверждаю";

pub fn make_agreement_keyboard() -> KeyboardMarkup {
    let buttons = vec![vec![KeyboardButton::new(AGREEMENT_BTN)]];
    KeyboardMarkup::new(buttons)
        .resize_keyboard()
        .one_time_keyboard()
}

pub const YES_BTN: &'static str = "Да";
pub const BACK_BTN: &'static str = "Назад";

pub fn make_yes_back_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(YES_BTN),
        KeyboardButton::new(BACK_BTN),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_citizenship_keyboard() -> KeyboardMarkup {
    let countries = vec![
        "Таджикистан",
        "Узбекистан",
        "Казахстан",
        "Кыргызстан",
        "Армения",
        "Беларусь",
        "Украина",
        "Другое",
    ];

    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];
    for chunk in countries.chunks(3) {
        keyboard.push(chunk.iter().map(|&c| KeyboardButton::new(c)).collect());
    }

    KeyboardMarkup::new(keyboard)
        .resize_keyboard()
        .one_time_keyboard()
}

pub const FIELD_NAME_LAT_BTN: &'static str = "Имя на латинице";
pub const FIELD_NAME_CYR_BTN: &'static str = "Имя на кириллицe";
pub const FIELD_CITIZENSHIP_BTN: &'static str = "Гражданство";
pub const FIELD_ARRIVAL_DATE_BTN: &'static str = "Дата прибытия";

pub fn make_field_selection_keyboard() -> KeyboardMarkup {
    let buttons = vec![
        vec![
            KeyboardButton::new(FIELD_NAME_LAT_BTN),
            KeyboardButton::new(FIELD_NAME_CYR_BTN),
        ],
        vec![
            KeyboardButton::new(FIELD_CITIZENSHIP_BTN),
            KeyboardButton::new(FIELD_ARRIVAL_DATE_BTN),
        ],
    ];

    KeyboardMarkup::new(buttons)
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn service_to_str(s: &Service) -> &'static str {
    match s {
        Service::InitialRegistration => "Первичная регистрация",
        Service::Visa => "Получение визы",
        Service::Insurance => "Страховка",
        Service::VisaAndInsurance => "Виза и страховка",
        Service::RenewalOfRegistration => "Продление регистрации",
        Service::RenewalOfVisa => "Продление визы",
        Service::All => "Все услуги",
    }
}

pub fn service_from_str(s: &str) -> Option<Service> {
    match s {
        "Первичная регистрация" => Some(Service::InitialRegistration),
        "Получение визы" => Some(Service::Visa),
        "Страховка" => Some(Service::Insurance),
        "Виза и страховка" => Some(Service::VisaAndInsurance),
        "Продление регистрации" => Some(Service::RenewalOfRegistration),
        "Продление визы" => Some(Service::RenewalOfVisa),
        "Все услуги" => Some(Service::All),
        _ => None,
    }
}

pub fn make_service_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(
        Service::all()
            .chunks(2)
            .map(|chunk| {
                chunk
                    .into_iter()
                    .map(|s| KeyboardButton::new(service_to_str(s)))
                    .collect::<Vec<KeyboardButton>>()
            })
            .collect::<Vec<_>>(),
    )
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_days_keyboard_with_back(days: &[NaiveDate]) -> KeyboardMarkup {
    let mut buttons = days
        .chunks(4)
        .map(|chunk| {
            chunk
                .into_iter()
                .map(|day| KeyboardButton::new(day.format("%m.%d").to_string()))
                .collect::<Vec<KeyboardButton>>()
        })
        .collect::<Vec<_>>();
    buttons.push(vec![KeyboardButton::new(BACK_BTN)]);
    KeyboardMarkup::new(buttons)
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn make_slots_keyboard_with_back(slots: &HashMap<String, FreeSlotDTO>) -> KeyboardMarkup {
    let mut starts: Vec<_> = slots.keys().collect();
    starts.sort();

    let mut buttons = starts
        .chunks(3)
        .map(|chunk| {
            chunk
                .into_iter()
                .map(|s| KeyboardButton::new(*s))
                .collect::<Vec<KeyboardButton>>()
        })
        .collect::<Vec<_>>();
    buttons.push(vec![KeyboardButton::new(BACK_BTN)]);
    KeyboardMarkup::new(buttons)
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn make_cancel_inline_keyboard(slot_start: DateTime<Utc>) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Отменить запись",
        slot_start.to_string(),
    )]])
}
