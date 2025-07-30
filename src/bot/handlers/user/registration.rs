use crate::bot::handlers::fsm::HandlerResult;
use crate::bot::handlers::keyboards::{
    AGREEMENT_BTN, make_agreement_keyboard, make_citizenship_keyboard,
};
use crate::domain::Error;
use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, UserID, Username};
use crate::usecases::{CheckRegisteredUseCase, RegisterUserRequest, RegisterUserUseCase};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{UpdateHandler, dialogue};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{KeyboardRemove, ParseMode};

#[derive(BotCommands, Clone)]
#[command(description = "Команды регистрации")]
enum RegistrationCommand {
    #[command(rename = "start", description = "начать регистрацию")]
    Start,

    #[command(rename = "cancel", description = "отменить регистрацию")]
    Cancel,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum RegistrationState {
    #[default]
    Start,
    AwaitingPDAgreement,
    AwaitingFullNameLat,
    AwaitingFullNameCyr(OnlyLatin),
    AwaitingCitizenship(OnlyLatin, OnlyCyrillic),
    AwaitingOtherCitizenship(OnlyLatin, OnlyCyrillic),
    AwaitingArrivalDate(OnlyLatin, OnlyCyrillic, Citizenship),
}

pub type RegistrationDialogue = Dialogue<RegistrationState, InMemStorage<RegistrationState>>;

async fn handle_start_command(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
    use_case: CheckRegisteredUseCase,
) -> HandlerResult {
    let registered = use_case.is_registered(UserID::new(msg.chat.id.0)).await?;
    if registered {
        bot.send_message(
            msg.chat.id,
            "🔹 <b>Уже зарегистрированы!</b>\n\
            Доступные команды:\n\
            /view – посмотреть свои данные;\n\
            /update – изменить данные;\n\
            /reserve – записаться на услугу.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id,
         "🌟 <b>Добро пожаловать</b>!\n\
          Для работы с ботом необходимо ваше согласие на обработку персональных данных в соответствии с 152-ФЗ РФ."
    )
        .parse_mode(ParseMode::Html)
        .reply_markup(make_agreement_keyboard())
        .await?;
    dialogue
        .update(RegistrationState::AwaitingPDAgreement)
        .await?;
    Ok(())
}

async fn receive_pd_agreement(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if text == AGREEMENT_BTN {
                bot.send_message(
                    msg.chat.id,
                    "✏️ <b>Введите ФИО латиницей</b>\n\
                     Пример: <i>Ivanov Ivan Ivanovich</i>",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                dialogue
                    .update(RegistrationState::AwaitingFullNameLat)
                    .await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ <b>Требуется подтверждение!</b>\n\
                     Пожалуйста, подтвердите согласие на обработку персональных данных, чтобы продолжить.",
                )
                    .parse_mode(ParseMode::Html)
                .reply_markup(make_agreement_keyboard())
                .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .reply_markup(make_agreement_keyboard())
                .await?;
        }
    }
    Ok(())
}

async fn receive_full_name_lat(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match OnlyLatin::new(text) {
            Ok(name) => {
                bot.send_message(
                    msg.chat.id,
                    "✏️ <b>Введите ФИО кириллицей</b>\n\
                     Пример: <i>Иванов Иван Иванович</i>",
                )
                .await?;
                dialogue
                    .update(RegistrationState::AwaitingFullNameCyr(name))
                    .await?;
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
    dialogue: RegistrationDialogue,
    full_name_lat: OnlyLatin,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match OnlyCyrillic::new(text) {
            Ok(full_name_cyr) => {
                let keyboard = make_citizenship_keyboard();
                bot.send_message(msg.chat.id, "🌍 <b>Выберите гражданство</b>")
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
                dialogue
                    .update(RegistrationState::AwaitingCitizenship(
                        full_name_lat,
                        full_name_cyr,
                    ))
                    .await?;
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
    dialogue: RegistrationDialogue,
    (full_name_lat, full_name_cyr): (OnlyLatin, OnlyCyrillic),
) -> HandlerResult {
    let text = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(
                msg.chat.id,
                "❌ <b>Ошибка ввода</b>\n\
                Используйте клавиатуру для ввода.",
            )
            .parse_mode(ParseMode::Html)
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
            bot.send_message(msg.chat.id, "🌍 <b>Введите гражданство</b>")
                .parse_mode(ParseMode::Html)
                .await?;
            dialogue
                .update(RegistrationState::AwaitingOtherCitizenship(
                    full_name_lat,
                    full_name_cyr,
                ))
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
            .await?;
            return Ok(());
        }
    };

    bot.send_message(
        msg.chat.id,
        "📅 <b>Введите дату прибытия</b>\n\
        В формате ДД.ММ.ГГГГ",
    )
    .parse_mode(ParseMode::Html)
    .await?;
    dialogue
        .update(RegistrationState::AwaitingArrivalDate(
            full_name_lat,
            full_name_cyr,
            citizenship,
        ))
        .await?;
    Ok(())
}

async fn receive_other_citizenship(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
    (full_name_lat, full_name_cyr): (OnlyLatin, OnlyCyrillic),
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
    bot.send_message(
        msg.chat.id,
        "📅 <b>Введите дату прибытия</b>\n\
        В формате ДД.ММ.ГГГГ",
    )
    .parse_mode(ParseMode::Html)
    .await?;
    dialogue
        .update(RegistrationState::AwaitingArrivalDate(
            full_name_lat,
            full_name_cyr,
            citizenship,
        ))
        .await?;
    Ok(())
}

async fn receive_arrival_date(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
    (full_name_lat, full_name_cyr, citizenship): (OnlyLatin, OnlyCyrillic, Citizenship),
    use_case: RegisterUserUseCase,
) -> HandlerResult {
    let date_str = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
            return Ok(());
        }
    };

    match NaiveDate::parse_from_str(date_str, "%d.%m.%Y") {
        Ok(arrival_date) => {
            let user = RegisterUserRequest {
                id: UserID::new(msg.chat.id.0),
                username: Username::new(msg.chat.username().unwrap_or_default().to_string()),
                full_name_lat,
                full_name_cyr,
                citizenship,
                arrival_date,
            };
            match use_case.register(user).await {
                Ok(_) => {
                    bot.send_message(
                        msg.chat.id,
                        "🎉 <b>Регистрация завершена!</b>\n\
                        Доступные команды:\n\
                        /view – посмотреть свои данные\n\
                        /update – изменить данные;\n\
                        /reserve – записаться на услугу.",
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                Err(e) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "❌ <b>Ошибка регистрации!</b>\n\
                            {}",
                            e
                        ),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
            }
            dialogue.exit().await?;
        }
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                "❌ <b>Неверный формат</b>\n\
                Введите дату в формате ДД.ММ.ГГГГ.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

async fn handle_cancel_command(
    bot: Bot,
    msg: Message,
    dialogue: RegistrationDialogue,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "🚫 Операция отменена")
        .reply_markup(KeyboardRemove::new())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

pub fn registration_schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<RegistrationCommand, _>()
        .branch(case![RegistrationCommand::Start].endpoint(handle_start_command))
        .branch(case![RegistrationCommand::Cancel].endpoint(handle_cancel_command));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![RegistrationState::AwaitingPDAgreement].endpoint(receive_pd_agreement))
        .branch(case![RegistrationState::AwaitingFullNameLat].endpoint(receive_full_name_lat))
        .branch(
            case![RegistrationState::AwaitingFullNameCyr(full_name_lat)]
                .endpoint(receive_full_name_cyr),
        )
        .branch(
            case![RegistrationState::AwaitingCitizenship(
                full_name_lat,
                full_name_cyr
            )]
            .endpoint(receive_citizenship),
        )
        .branch(
            case![RegistrationState::AwaitingOtherCitizenship(
                full_name_lat,
                full_name_cyr
            )]
            .endpoint(receive_other_citizenship),
        )
        .branch(
            case![RegistrationState::AwaitingArrivalDate(
                full_name_lat,
                full_name_cyr,
                citizenship
            )]
            .endpoint(receive_arrival_date),
        );

    dialogue::enter::<Update, InMemStorage<RegistrationState>, RegistrationState, _>()
        .branch(message_handler)
}
