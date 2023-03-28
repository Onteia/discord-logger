
use std::fs;
use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;


struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        
        //ignore messages from MessageLogger
        if msg.author.bot && msg.author.name == "MessageLogger" {
            return;
        }

        msg.channel_id.send_message(
            &ctx, 
            |reply| {
                reply.content(msg.content)
            })
            .await
            .unwrap();
    }


}

const DISCORD_AUTH_PATH: &'static str = "discord.auth";

#[tokio::main]
async fn main() {   
    //get the token from file
    let token = fs::read_to_string(DISCORD_AUTH_PATH).expect("could not read discord token file!");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    //build the client
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

}
