use crate::bot::handlers::fsm::HandlerResult;
use crate::bot::handlers::keyboards::{
    BACK_BTN, YES_BTN, make_cancel_inline_keyboard, make_days_keyboard_with_back,
    make_service_keyboard, make_slots_keyboard_with_back, make_yes_back_keyboard, service_from_str,
    service_to_str,
};
use crate::domain::Error;
use crate::domain::models::{Service, UserID};
use crate::usecases::{
    CancelReservationUseCase, CheckDeadlineUseCase, CheckRegisteredUseCase,
    DaysWithFreeSlotsUseCase, FreeSlotDTO, FreeSlotsUseCase, ReserveSlotUseCase,
};
use chrono::{DateTime, Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{UpdateHandler, dialogue};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

#[derive(BotCommands, Clone)]
#[command(description = "Команды записи")]
enum SlotsCommand {
    #[command(rename = "reserve", description = "записаться на получение услуги")]
    Reserve,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum SlotsState {
    #[default]
    Start,
    AwaitingServiceType,
    AwaitingDay(Service, Vec<NaiveDate>),
    AwaitingSlot(Service, Vec<NaiveDate>, HashMap<String, FreeSlotDTO>),
    AwaitingApproval(
        Service,
        Vec<NaiveDate>,
        HashMap<String, FreeSlotDTO>,
        FreeSlotDTO,
    ),
    AwaitingApprovalOfCancel,
}

pub type SlotsDialogue = Dialogue<SlotsState, InMemStorage<SlotsState>>;

async fn handle_reserve_command(
    bot: Bot,
    msg: Message,
    dialogue: SlotsDialogue,
    check_registered: CheckRegisteredUseCase,
) -> HandlerResult {
    let registered = check_registered
        .is_registered(UserID::new(msg.chat.id.0))
        .await?;
    if !registered {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Сначала зарегистрируйтесь!</b>\n\
             Введите /start для начала.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }
    bot.send_message(
        msg.chat.id,
        "🔹 <b>Выберите тип услуги</b>\n\
        Обратите внимание, что в зависимости от типа отличаются дни и срок оказания услуги.\n\
        В пятницу доступна только услуга \"Консультация по РВПО/ВНЖ\".\n\
        \"Первичная регистрация\", \"Виза\" и \"Все услуги\" имеют следующие сроки начиная от \
        времени прибытия:\n\
        - Таджикистан, Узбекистан - 15 дней;\n\
        - Казахстан, Кыргызстан, Армения - 30 дней;\n\
        - Беларусь, Украина - 90 дней;\n\
        - Другие страны - 7 дней.",
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(make_service_keyboard(Service::all()))
    .await?;
    dialogue.update(SlotsState::AwaitingServiceType).await?;
    Ok(())
}

async fn receive_service_type(
    bot: Bot,
    msg: Message,
    dialogue: SlotsDialogue,
    cd_use_case: CheckDeadlineUseCase,
    dfs_use_case: DaysWithFreeSlotsUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match service_from_str(text) {
            Some(service) => {
                let user_id = UserID::new(msg.chat.id.0);
                let ok = cd_use_case.check_deadline(user_id, service).await?;
                if ok {
                    let days = dfs_use_case.days_with_free_slots(user_id, service).await?;
                    if days.is_empty() {
                        bot.send_message(msg.chat.id, "😔 <b>Нет доступных дней для записи</b>")
                            .parse_mode(ParseMode::Html)
                            .await?;
                        dialogue.exit().await?;
                    } else {
                        bot.send_message(msg.chat.id, "📅 <b>Выберите удобный день</b>")
                            .parse_mode(ParseMode::Html)
                            .reply_markup(make_days_keyboard_with_back(&days))
                            .await?;
                        dialogue
                            .update(SlotsState::AwaitingDay(service, days))
                            .await?;
                    }
                } else {
                    bot.send_message(msg.chat.id, "⏳ <b>Срок подачи заявки истек</b>")
                        .parse_mode(ParseMode::Html)
                        .await?;
                }
            }
            None => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                    Используйте клавиатуру для ввода.",
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(make_service_keyboard(Service::all()))
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_service_keyboard(Service::all()))
                .await?;
        }
    }
    Ok(())
}

fn fetch_month_and_date(s: &str) -> Option<(u32, u32)> {
    let v = s
        .splitn(2, '.')
        .map(|s| s.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>();
    v.map(|v| (v[0], v[1]))
}

fn make_slots_map(slots: Vec<FreeSlotDTO>) -> HashMap<String, FreeSlotDTO> {
    HashMap::from_iter(slots.into_iter().map(|slot| {
        (
            format!(
                "{} - {}",
                slot.start.format("%H:%M"),
                slot.end.format("%H:%M")
            ),
            slot,
        )
    }))
}

async fn receive_day(
    bot: Bot,
    msg: Message,
    dialogue: SlotsDialogue,
    (service, days): (Service, Vec<NaiveDate>),
    use_case: FreeSlotsUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(BACK_BTN) => {
            bot.send_message(msg.chat.id, "Выберите тип услуги")
                .reply_markup(make_service_keyboard(Service::all()))
                .await?;
            dialogue.update(SlotsState::AwaitingServiceType).await?;
        }
        Some(text) => match fetch_month_and_date(text) {
            Some((month, day)) => {
                match days.iter().find(|&d| d.month() == month && d.day() == day) {
                    Some(date) => {
                        let slots = use_case.free_slots(*date, service).await?;
                        let slots = make_slots_map(slots);
                        bot.send_message(msg.chat.id, "⏰ <b>Выберите доступный слот</b>")
                            .parse_mode(ParseMode::Html)
                            .reply_markup(make_slots_keyboard_with_back(&slots))
                            .await?;
                        dialogue
                            .update(SlotsState::AwaitingSlot(service, days, slots))
                            .await?;
                    }
                    None => {
                        bot.send_message(
                            msg.chat.id,
                            "❌ <b>Ошибка ввода</b>\n\
                             Используйте клавиатуру для ввода.",
                        )
                        .parse_mode(ParseMode::Html)
                        .reply_markup(make_days_keyboard_with_back(&days))
                        .await?;
                    }
                }
            }
            None => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                    Используйте клавиатуру для ввода.",
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(make_days_keyboard_with_back(&days))
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_days_keyboard_with_back(&days))
                .await?;
        }
    }
    Ok(())
}

async fn receive_slot(
    bot: Bot,
    msg: Message,
    dialogue: SlotsDialogue,
    (service, days, slots): (Service, Vec<NaiveDate>, HashMap<String, FreeSlotDTO>),
) -> HandlerResult {
    match msg.text() {
        Some(BACK_BTN) => {
            bot.send_message(msg.chat.id, "Выберите доступный день для получения услуги")
                .reply_markup(make_days_keyboard_with_back(&days))
                .await?;
            dialogue
                .update(SlotsState::AwaitingDay(service, days))
                .await?;
        }
        Some(text) => match slots.get(text) {
            Some(slot) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "❔ <b>Подтверждаете запись?</b>\n\
                        Услуга: «{}»\n\
                        Время: {}",
                        service_to_str(&service),
                        slot.start.format("%m.%d %H:%M"),
                    ),
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(make_yes_back_keyboard())
                .await?;
                dialogue
                    .update(SlotsState::AwaitingApproval(
                        service,
                        days,
                        slots.clone(),
                        slot.clone(),
                    ))
                    .await?;
            }
            None => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                    Используйте клавиатуру для ввода.",
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(make_slots_keyboard_with_back(&slots))
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_days_keyboard_with_back(&days))
                .await?;
        }
    }
    Ok(())
}

async fn receive_approval(
    bot: Bot,
    msg: Message,
    dialogue: SlotsDialogue,
    (service, days, slots, slot): (
        Service,
        Vec<NaiveDate>,
        HashMap<String, FreeSlotDTO>,
        FreeSlotDTO,
    ),
    use_case: ReserveSlotUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(BACK_BTN) => {
            bot.send_message(msg.chat.id, "Выберите один из предложенных слотов")
                .reply_markup(make_slots_keyboard_with_back(&slots))
                .await?;
            dialogue
                .update(SlotsState::AwaitingSlot(service, days, slots))
                .await?;
        }
        Some(YES_BTN) => {
            let user_id = UserID::new(msg.chat.id.0);
            match use_case.reserve_slot(user_id, slot.start, service).await {
                Ok(_) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "✅ <b>Запись успешно создана!</b>\n\
                            Услуга: «{}»\n\
                            Время: {}",
                            service_to_str(&service),
                            slot.start.format("%m.%d %H:%M"),
                        ),
                    )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(make_cancel_inline_keyboard(slot.start))
                    .await?;
                    dialogue.exit().await?;
                }
                Err(Error::SlotNotFound) => {
                    bot.send_message(
                        msg.chat.id,
                        "😕 <b>Этот слот уже занят</b>\n\
                        Попробуйте снова: /reserve",
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                    dialogue.exit().await?;
                }
                Err(Error::SlotAlreadyReserved(_)) => {
                    bot.send_message(
                        msg.chat.id,
                        "❌ <b>Ошибка бронирования</b>\n\
                        Слот уже забронирован Вами. Попробуйте снова: /reserve",
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                    dialogue.exit().await?;
                }
                Err(e) => return Err(e),
            }
        }
        Some(_) => {
            bot.send_message(
                msg.chat.id,
                "❌ <b>Ошибка ввода</b>\n\
                    Используйте клавиатуру для ввода.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(make_yes_back_keyboard())
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_yes_back_keyboard())
                .await?;
        }
    }
    Ok(())
}

async fn handle_cancel_callback(
    bot: Bot,
    q: CallbackQuery,
    use_case: CancelReservationUseCase,
) -> HandlerResult {
    let user_id = UserID::new(q.from.id.0 as i64);
    #[allow(clippy::collapsible_if)] // Нельзя, так как && в этом месте есть unstable option
    if let Some(data) = q.data {
        if let Ok(date) = DateTime::from_str(&data) {
            use_case.cancel_reservation(user_id, date).await?;
            bot.answer_callback_query(q.id).await?;
            if let Some(msg) = q.message {
                bot.edit_message_reply_markup(msg.chat().id, msg.id())
                    .await?;
                bot.send_message(
                    msg.chat().id,
                    format!(
                        "🚫 <b>Запись на {} отменена</b>",
                        date.format("%m.%d %H:%M"),
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        }
    }
    Ok(())
}

pub fn slots_schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<SlotsCommand, _>()
        .branch(case![SlotsCommand::Reserve].endpoint(handle_reserve_command));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![SlotsState::AwaitingServiceType].endpoint(receive_service_type))
        .branch(case![SlotsState::AwaitingDay(service, days)].endpoint(receive_day))
        .branch(case![SlotsState::AwaitingSlot(service, days, slots)].endpoint(receive_slot))
        .branch(
            case![SlotsState::AwaitingApproval(service, days, slots, slot)]
                .endpoint(receive_approval),
        );

    let callback_handler = Update::filter_callback_query().endpoint(handle_cancel_callback);

    dialogue::enter::<Update, InMemStorage<SlotsState>, SlotsState, _>()
        .branch(message_handler)
        .branch(callback_handler)
}
