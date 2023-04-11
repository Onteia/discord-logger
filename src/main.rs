
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serenity::async_trait;
use serenity::prelude::*;
use serenity::Client;
use serenity::model::prelude::Ready;
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serde::{Serialize, Deserialize};


const DISCORD_AUTH_PATH: &'static str = "discord.auth";
const JSON_PATH: &'static str = "./servers.json";
const LOGGER_NAME: &'static str = "MessageLogger";

/*TODO: 
    properly log embeds
    log message editing
    possibly proper gif rendering
        because the gifs are cached and they could get removed from the cdn
*/

#[derive(Serialize, Deserialize, Debug)]
struct SaveMap {
    #[serde(flatten)]
    map: HashMap<String, u64>
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    //when MessageLogger starts
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        let activity = Activity::playing("!setuplogging");
        ctx.set_activity(activity).await;
    }
    
    // when a user sends a message
    async fn message(&self, ctx: Context, msg: Message) {

        //ignore messages from MessageLogger
        if msg.author.bot && msg.author.name == LOGGER_NAME {
            return;
        }

        let g_id = msg.guild_id
            .expect("unable to get the guild_id!")
            .to_string();

        //read from json file
        let contents = fs::read_to_string(JSON_PATH)
            .expect("unable to read the json file!");
        
        let mut save_map: HashMap<String, u64>;
        //if the json file is empty, initialize the hash map
        //deserialize existing json file otherwise
        if contents == "" {
            save_map = HashMap::new();
        } else {
            save_map = serde_json::from_str::<SaveMap>(&contents)
                .expect("unable to convert the json file to a SaveMap!").map;
        }

        if msg.content.starts_with("!setuplogging") {
            //get current channel id
            let c_id = *msg.channel_id.as_u64();
            
            //update the save_map with the new server,channel pair
            save_map.insert(g_id, c_id);
            
            //serialize and write to the json file
            let serialized = serde_json::to_string(&save_map)
                .expect("!setuplogging: unable to serialize the save_map!");
            fs::write(JSON_PATH, &serialized)
                .expect("!setuplogging: unable to write the serializable object to the json file!");

            //send confirmation message
            msg.channel_id
                .send_message(
                    &ctx, 
                    |reply| {
                        reply.content("logging has been successfully set up for this channel!")
                    })
                .await
                .unwrap();

            return;

        }

        //ignore messages if logging not set up
        if !save_map.contains_key(&g_id) {
            return;
        }

        //get the channel id associated with the guild id
        let c_id = *save_map.get(&g_id)
            .expect("unable to get the channel id from the hash map!");
        //turn the c_id into a guild channel
        let g_channel = ctx.http.get_channel(c_id).await
            .expect("unable to get the channel!")
            .guild().expect("unable to get the guild channel!");
        
        let author = &msg.author.name;
        let link = msg.link();
        
        //get the channel name to format it as: `#channel_name` in the embed
        let guild_channel = msg.channel(&ctx).await
            .expect("unable to get the channel from the message!")
            .guild().expect("unable to get the guild from the channel!");
        let channel_name = "#".to_owned() + guild_channel.name();

        g_channel.send_message(
            &ctx, 
            |reply| {
                reply.add_embed(|e| {
                    e.title(author);
                    e.url(link);
                    e.description(channel_name);
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

    //check existence of the json file
    let json_exists = Path::try_exists(Path::new(JSON_PATH))
        .expect("unable to access the json file!");

    //create json file if it doesn't exist
    if !json_exists{ fs::write(JSON_PATH, "").expect("main(): unable to initialize json file!"); }

    setup_bot().await;

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
        println!("an error occurred while running the client: {:?}", why);
    }
}
