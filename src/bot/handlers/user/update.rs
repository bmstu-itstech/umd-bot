use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{UpdateHandler, dialogue};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{KeyboardRemove, ParseMode};

use crate::bot::handlers::fsm::HandlerResult;
use crate::bot::handlers::keyboards::{
    self, make_citizenship_keyboard, make_field_selection_keyboard,
};
use crate::domain::Error;
use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, UserID};
use crate::usecases::{CheckRegisteredUseCase, UpdateUserUseCase};

#[derive(BotCommands, Clone)]
#[command(description = "Команды профиля")]
enum UpdateCommand {
    #[command(rename = "update", description = "обновить данные")]
    Update,

    #[command(rename = "cancel", description = "отменить операцию")]
    Cancel,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum UpdateState {
    #[default]
    Start,
    AwaitingFieldSelection,
    AwaitingFullNameLat,
    AwaitingFullNameCyr,
    AwaitingCitizenship,
    AwaitingOtherCitizenship,
    AwaitingArrivalDate,
}

pub type UpdateDialogue = Dialogue<UpdateState, InMemStorage<UpdateState>>;

async fn handle_update_command(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: CheckRegisteredUseCase,
) -> HandlerResult {
    let registered = use_case.is_registered(UserID::new(msg.chat.id.0)).await?;
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
    bot.send_message(msg.chat.id, "🔄 <b>Что хотите обновить?</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(make_field_selection_keyboard())
        .await?;
    dialogue.update(UpdateState::AwaitingFieldSelection).await?;
    Ok(())
}

async fn receive_field_selection(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text {
            keyboards::FIELD_NAME_LAT_BTN => {
                bot.send_message(
                    msg.chat.id,
                    "✏️ <b>Введите новое ФИО латиницей</b>\n\
                     Пример: <i>Ivanov Ivan Ivanovich</i>",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                dialogue.update(UpdateState::AwaitingFullNameLat).await?;
            }
            keyboards::FIELD_NAME_CYR_BTN => {
                bot.send_message(
                    msg.chat.id,
                    "✏️ <b>Введите новое ФИО кириллицей</b>\n\
                     Пример: <i>Иванов Иван Иванович</i>",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                dialogue.update(UpdateState::AwaitingFullNameCyr).await?;
            }
            keyboards::FIELD_CITIZENSHIP_BTN => {
                bot.send_message(msg.chat.id, "🌍 <b>Выберите новое гражданство</b>")
                    .parse_mode(ParseMode::Html)
                    .reply_markup(make_citizenship_keyboard())
                    .await?;
                dialogue.update(UpdateState::AwaitingCitizenship).await?;
            }
            keyboards::FIELD_ARRIVAL_DATE_BTN => {
                bot.send_message(
                    msg.chat.id,
                    "📅 <b>Введите дату прибытия</b>\n\
                    В формате ДД.ММ.ГГГГ",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                dialogue.update(UpdateState::AwaitingArrivalDate).await?;
            }
            _ => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                    Используйте клавиатуру для ввода.",
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(make_field_selection_keyboard())
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_field_selection_keyboard())
                .await?;
        }
    }
    Ok(())
}

async fn receive_full_name_lat(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: UpdateUserUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match OnlyLatin::new(text) {
            Ok(name) => {
                use_case.update_name_lat(msg.chat.id.0, name).await?;
                bot.send_message(msg.chat.id, "✅ Имя на латинице изменено!")
                    .await?;
                dialogue.exit().await?;
            }
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                     Допустимы только латинские символы. Попробуйте еще раз.",
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
        }
    }
    Ok(())
}

async fn receive_full_name_cyr(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: UpdateUserUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match OnlyCyrillic::new(text) {
            Ok(name) => {
                use_case.update_name_cyr(msg.chat.id.0, name).await?;
                bot.send_message(msg.chat.id, "✅ Имя на кириллице изменено!")
                    .await?;
                dialogue.exit().await?;
            }
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "❌ <b>Ошибка ввода</b>\n\
                     Допустимы только кириллические символы. Попробуйте еще раз.",
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
        }
    }
    Ok(())
}

async fn receive_citizenship(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: UpdateUserUseCase,
) -> HandlerResult {
    let text = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_citizenship_keyboard())
                .await?;
            return Ok(());
        }
    };

    let citizenship = match text {
        "Таджикистан" => Citizenship::Tajikistan,
        "Узбекистан" => Citizenship::Uzbekistan,
        "Казахстан" => Citizenship::Kazakhstan,
        "Кыргызстан" => Citizenship::Kyrgyzstan,
        "Армения" => Citizenship::Armenia,
        "Беларусь" => Citizenship::Belarus,
        "Украина" => Citizenship::Ukraine,
        "Другое" => {
            bot.send_message(msg.chat.id, "🌍 <b>Выберите гражданство</b>")
                .parse_mode(ParseMode::Html)
                .reply_markup(KeyboardRemove::new())
                .await?;
            dialogue
                .update(UpdateState::AwaitingOtherCitizenship)
                .await?;
            return Ok(());
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "❌ <b>Ошибка ввода</b>\n\
                Используйте клавиатуру для ввода.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(make_citizenship_keyboard())
            .await?;
            return Ok(());
        }
    };
    use_case
        .update_citizenship(msg.chat.id.0, citizenship)
        .await?;
    bot.send_message(msg.chat.id, "✅ Гражданство изменено!")
        .reply_markup(KeyboardRemove::new())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_other_citizenship(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: UpdateUserUseCase,
) -> HandlerResult {
    let other = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
            return Ok(());
        }
    };
    let citizenship = Citizenship::Other(other.to_string());
    use_case
        .update_citizenship(msg.chat.id.0, citizenship)
        .await?;
    bot.send_message(msg.chat.id, "✅ Гражданство изменено!")
        .reply_markup(KeyboardRemove::new())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_arrival_date(
    bot: Bot,
    msg: Message,
    dialogue: UpdateDialogue,
    use_case: UpdateUserUseCase,
) -> HandlerResult {
    let date_str = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
            return Ok(());
        }
    };
    let arrival_date = match NaiveDate::parse_from_str(date_str, "%d.%m.%Y") {
        Ok(arrival_date) => arrival_date,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                "❌ <b>Неверный формат</b>\n\
                Введите дату в формате ДД.ММ.ГГГГ.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };
    use_case
        .update_arrival_date(msg.chat.id.0, arrival_date)
        .await?;
    bot.send_message(msg.chat.id, "✅ Дата прибытия изменена!")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn handle_cancel_command(bot: Bot, msg: Message, dialogue: UpdateDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "❌ Текущая операция отменена")
        .reply_markup(KeyboardRemove::new())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

pub fn update_schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<UpdateCommand, _>()
        .branch(case![UpdateCommand::Update].endpoint(handle_update_command))
        .branch(case![UpdateCommand::Cancel].endpoint(handle_cancel_command));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![UpdateState::AwaitingFieldSelection].endpoint(receive_field_selection))
        .branch(case![UpdateState::AwaitingFullNameLat].endpoint(receive_full_name_lat))
        .branch(case![UpdateState::AwaitingFullNameCyr].endpoint(receive_full_name_cyr))
        .branch(case![UpdateState::AwaitingCitizenship].endpoint(receive_citizenship))
        .branch(case![UpdateState::AwaitingOtherCitizenship].endpoint(receive_other_citizenship))
        .branch(case![UpdateState::AwaitingArrivalDate].endpoint(receive_arrival_date));

    dialogue::enter::<Update, InMemStorage<UpdateState>, UpdateState, _>().branch(message_handler)
}
