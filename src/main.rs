use std::env;
use regex::Regex;
use url::Url;
use reqwest::blocking::{Client, Response};
use teloxide::prelude::*;
use teloxide::types::{Message, ParseMode};
use teloxide::utils::command::BotCommand;

#[derive(BotCommand)]
#[command(rename = "lowercase")]
enum Command {
    #[command(description = "Start command")]
    Start,
}

async fn start_handler(message: Message, cx: UpdateWithCx<Bot>) -> ResponseResult<()> {
    cx.answer_str("aaaa").await?;
    Ok(())
}

fn find_b23_urls(content: &str) -> Vec<Url> {
    let re = Regex::new(r"(https?://\S+)").unwrap();
    let list_of_url_strs: Vec<&str> = re.find_iter(content).map(|m| m.as_str()).collect();
    let list_of_url: Vec<Url> = list_of_url_strs.iter().filter_map(|&url| Url::parse(url).ok()).collect();
    let filtered_b23_url: Vec<Url> = list_of_url.into_iter().filter(|url| url.host_str() == Some("b23.tv")).collect();
    filtered_b23_url
}

fn access_b23_url_and_return_real_url(url: &Url) -> String {
    let client = Client::new();
    let res = client.get(url.clone()).send().unwrap();
    let real_url = res.url().clone();
    let mut r = real_url.clone();
    r.set_fragment(None);
    r.into_string()
}

async fn remove_b23(message: Message, cx: UpdateWithCx<Bot>) -> ResponseResult<()> {
    let filtered_b23_url = find_b23_urls(&message.text.unwrap_or_default());
    if !filtered_b23_url.is_empty() {
        let user_name = message.from.as_ref().unwrap().first_name.clone();
        let url_list: Vec<String> = filtered_b23_url.iter().map(|url| access_b23_url_and_return_real_url(url)).collect();
        let urls_str = url_list.join("\n");
        let content = format!("{} 分享了B站链接为:\n{}", user_name, urls_str);
        cx.reply_to(content).await?;
        cx.delete_message().await?;
    }
    Ok(())
}

async fn complete_bv(message: Message, cx: UpdateWithCx<Bot>) -> ResponseResult<()> {
    if let Some(text) = message.text {
        if text.starts_with("BV") {
            cx.reply_to(format!("https://b23.tv/{}", text)).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <bot_token>");
        return;
    }
    let bot_token = &args[1];
    let bot = Bot::new(bot_token);
    let bot_name = bot.get_me().await.unwrap().user.username.unwrap();
    let commands = vec![Command::start()];
    let bot = bot
        .name(bot_name)
        .commands(commands)
        .parse_mode(ParseMode::MarkdownV2);
    let bot = Dispatcher::new(bot);
    bot.message(Command::start().filter(|_| true), start_handler)
        .await;
    bot.message(filter::Text, remove_b23).await;
    bot.message(filter::Text, complete_bv).await;
    bot.polling().await.unwrap();
}
