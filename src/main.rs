
use std::fs;
use serenity::async_trait;
use serenity::prelude::*;
use serenity::Client;
use serenity::model::prelude::Ready;
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;


const DISCORD_AUTH_PATH: &'static str = "discord.auth";

struct Handler;

struct Server {
    guild_id: String,
    channel_id: String
}

#[async_trait]
impl EventHandler for Handler {

    //when MessageLogger starts
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        let activity = Activity::playing("!setuplogging");
        ctx.set_activity(activity);
    }

    // when a user sends a message
    async fn message(&self, ctx: Context, msg: Message) {
        
        //ignore messages from MessageLogger
        if msg.author.bot && msg.author.name == "MessageLogger" {
            return;
        }

        if msg.content.starts_with("!setuplogging") {
            
            //save channel id and associate with guild id
            println!("guildID: {} \nchannelID: {}", msg.guild_id.unwrap(), msg.channel_id);

            

            return;

        }




        //ignore messages if logging not set up

        //read log file associated with channel id
        
        //channel to send the log message to
        //let log_channel: ChannelId;

        let author = &msg.author.name;
        let link = &msg.link();


        msg.channel_id.send_message(
            &ctx, 
            |reply| {
                reply.add_embed(|e| {
                    e.title(author);
                    e.url(link);
                    e.description("#channel");
                    e.field("posted:", msg.content, false);
                    e.timestamp(msg.timestamp)
                })
            })
            .await
            .unwrap();
    }


}



#[tokio::main]
async fn main() {   
    
    setup_bot().await;

    //read json file

    //get info of discord server and log channel pairs

}

async fn setup_bot() {
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
