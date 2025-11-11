use crate::bot::handlers::fsm::HandlerResult;
use crate::domain::Error;
use crate::domain::models::UserID;
use crate::usecases::GetUserUseCase;
use teloxide::dispatching::UpdateHandler;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

#[derive(BotCommands, Clone)]
#[command(description = "Команды профиля")]
enum ViewCommand {
    #[command(rename = "view", description = "Показать данные обо мне")]
    View,
}

pub async fn handle_view_command(
    bot: Bot,
    msg: Message,
    use_case: GetUserUseCase,
) -> HandlerResult {
    match use_case.user(UserID::new(msg.chat.id.0)).await {
        Ok(user) => {
            let text = format!(
                "📋 <b>Ваши данные</b>\n\
                👤 Имя (лат): {}\n\
                👤 Имя (кир): {}\n\
                🌍 Гражданство: {}\n\
                📅 Дата прибытия: {}",
                user.full_name_lat.as_str(),
                user.full_name_cyr.as_str(),
                user.citizenship.as_str(),
                user.arrival_date.format("%d.%m.%Y"),
            );
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(Error::UserNotFound(_)) => {
            bot.send_message(
                msg.chat.id,
                "❌ Вы еще не зарегистрированы. Используйте /start для регистрации.",
            )
            .await?;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

pub fn view_schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<ViewCommand, _>()
        .branch(case![ViewCommand::View].endpoint(handle_view_command));

    Update::filter_message().branch(command_handler)
}
