use chrono::NaiveDate;
use csv::Writer;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{UpdateHandler, dialogue};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};

use crate::bot::handlers::fsm::HandlerResult;
use crate::bot::handlers::keyboards::service_to_str;
use crate::domain::Error;
use crate::domain::models::UserID;
use crate::usecases::{CheckAdminUseCase, ReservationDTO, ReservationsUseCase};

#[derive(BotCommands, Clone)]
#[command(description = "Команды записи")]
enum AdminCommand {
    #[command(rename = "table", description = "получить таблицу с записями на день")]
    Table,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum AdminState {
    #[default]
    Start,
    AwaitingDate,
}

pub type AdminDialogue = Dialogue<AdminState, InMemStorage<AdminState>>;

async fn handle_table_command(
    bot: Bot,
    msg: Message,
    dialogue: AdminDialogue,
    use_case: CheckAdminUseCase,
) -> HandlerResult {
    let user_id = UserID::new(msg.chat.id.0);
    if !use_case.is_admin(user_id).await? {
        bot.send_message(msg.chat.id, "⛔ <b>Доступ запрещен</b>")
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }
    bot.send_message(
        msg.chat.id,
        "📅 <b>Введите дату</b>\n\
        В формате ДД.ММ.ГГГГ",
    )
    .parse_mode(ParseMode::Html)
    .await?;
    dialogue.update(AdminState::AwaitingDate).await?;
    Ok(())
}

async fn receive_date(
    bot: Bot,
    msg: Message,
    dialogue: AdminDialogue,
    use_case: ReservationsUseCase,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
            Ok(date) => {
                let reservations = use_case.reservations(date).await?;
                let csv_data = generate_csv(&reservations)?;
                let file_name = format!("slots_{}.csv", date.format("%Y-%m-%d"));
                let input_file = InputFile::memory(csv_data).file_name(file_name);
                bot.send_document(msg.chat.id, input_file).await?;
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
        },
        None => {
            bot.send_message(msg.chat.id, "📝 Введите текстовое сообщение")
                .await?;
        }
    }
    Ok(())
}

fn generate_csv(rs: &[ReservationDTO]) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    // UTF-8 BOM
    buffer.extend_from_slice(&[0xEF, 0xBB, 0xBF]);

    let mut writer = Writer::from_writer(buffer);

    writer
        .write_record([
            "#",
            "Начало",
            "Конец",
            "Услуга",
            "Telegram",
            "ФИО (лат)",
            "ФИО (кир)",
            "Гражданство",
            "Дата прибытия",
        ])
        .map_err(|err| Error::Other(err.into()))?;

    for (i, r) in rs.iter().enumerate() {
        writer
            .write_record(&[
                format!("{}", i + 1),
                r.slot_start.format("%H:%M").to_string(),
                r.slot_end.format("%H:%M").to_string(),
                service_to_str(&r.service).to_string(),
                format!("t.me/{}/", r.username),
                r.user_name_lat.clone(),
                r.user_name_cyr.clone(),
                r.citizenship.clone().into(),
                r.arrival_date.format("%d.%m.%Y").to_string(),
            ])
            .map_err(|err| Error::Other(err.into()))?;
    }

    writer.flush().map_err(|err| Error::Other(err.into()))?;
    let res = writer
        .into_inner()
        .map_err(|err| Error::Other(err.into()))?;
    Ok(res)
}

pub fn admin_schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<AdminCommand, _>()
        .branch(case![AdminCommand::Table].endpoint(handle_table_command));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![AdminState::AwaitingDate].endpoint(receive_date));

    dialogue::enter::<Update, InMemStorage<AdminState>, AdminState, _>().branch(message_handler)
}
