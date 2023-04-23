
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::collections::HashMap;
use serenity::async_trait;
use serenity::model::Timestamp;
use serenity::model::prelude::MessageUpdateEvent;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionType;
use serenity::model::guild::UnavailableGuild;
use serenity::prelude::*;
use serenity::Client;
use serenity::model::permissions::Permissions;
use serenity::model::prelude::Ready;
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serenity::utils::Color;
use serde::{Serialize, Deserialize};
use serde_json::Error;


const DISCORD_AUTH_PATH: &'static str = "discord.auth";
const JSON_PATH: &'static str = "./servers.json";
//Clueless just assume tag won't change 
const LOGGER_TAG: &'static str = "MessageLogger#0584";

const INIT_LOG: &'static str = "setuplogging";
const END_LOG: &'static str = "removelogging";


/*TODO: 

    move icon to be like an author icon

    properly log embeds
    possibly proper gif rendering
        because the gifs are cached and they could get removed from the cdn
    proper attachment logging

    ~delete log of server when bot is kicked

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
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let activity = Activity::playing("/".to_owned() + INIT_LOG);
        ctx.set_activity(activity).await;
    
        sync_json(data_about_bot.guilds).unwrap();

        let _init_log = Command::create_global_application_command(
            &ctx, 
            |command| {
                command.name(INIT_LOG);
                command.description("setup logging for this channel");
                command.default_member_permissions(Permissions::MANAGE_GUILD)
            }
        ).await;

        let _end_log = Command::create_global_application_command(
            &ctx, 
            |command| {
                command.name(END_LOG);
                command.description("remove logging for your server");
                command.default_member_permissions(Permissions::MANAGE_GUILD)
            }
        ).await;

    }
    
    //handle interactions
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        
        //handle slash commands
        if interaction.kind() == InteractionType::ApplicationCommand {
            let slash_command = interaction.application_command()
                .expect("interaction_create(): unable to convert the interaction 
                to an application command!");
            
            let command_name = slash_command.data.name.clone();

            if command_name == INIT_LOG {
                //get current channel id
                let c_id = *slash_command.channel_id.as_u64();
                let g_id_str = slash_command.guild_id
                    .expect((INIT_LOG.to_owned() + ": unable to get the guild_id!").as_str())
                    .to_string();

                //update the save_map with the new server,channel pair
                let mut save_map = read_json()
                    .expect((INIT_LOG.to_owned() + ": unable to read from json!").as_str());
                save_map.insert(g_id_str, c_id);
                
                write_json(&save_map)
                    .expect((INIT_LOG.to_owned() + ": unable to write to json file!").as_str());

                slash_command
                    .create_interaction_response(
                        &ctx, 
                        |reply| {
                            reply.interaction_response_data(|message| {
                                message.content("logging has been successfully set up for this channel!")
                        })
                    })
                    .await
                    .unwrap();

                return;
            }
       
            if command_name == END_LOG {
                let g_id_str: String = slash_command.guild_id
                    .expect((END_LOG.to_owned() + ": unable to get the guild_id!").as_str())
                    .to_string();
                
                //remove from map and update json
                match delete_entry(&g_id_str) {
                    //send success message
                    Some(_id) => {
                        slash_command
                            .create_interaction_response(
                                &ctx, 
                                |reply| {
                                    reply.interaction_response_data(|message| {
                                        message.content("logging successfully stopped for this server!")
                                })
                            })
                            .await
                            .unwrap();
                    },
                    //send error message
                    None => {
                        slash_command
                            .create_interaction_response(
                                &ctx, 
                                |reply| {
                                    reply.interaction_response_data(|message| {
                                        message.content("logging has not been set up yet for your server!")
                                })
                            })
                            .await
                            .unwrap();
                    }
                }
                return;
            }
       
        }

    }

    //when a user sends a message
    async fn message(&self, ctx: Context, msg: Message) {
        
        let author = msg.author.clone();

        //ignore messages from MessageLogger
        if author.bot && author.tag().as_str() == LOGGER_TAG {
            return;
        }

        let g_id = msg.guild_id
            .expect("message(): unable to get the guild_id!");
        let g_id_str = g_id.to_string();

        let save_map = read_json()
            .expect("message(): unable to read from json!");
        
        //ignore messages if logging not set up
        if !save_map.contains_key(&g_id_str) {
            return;
        }


        //get the channel id associated with the guild id
        let c_id = *save_map.get(&g_id_str)
            .expect("message(): unable to get the channel id from the hash map!");
        //turn the c_id into a guild channel
        let log_channel = ctx.http.get_channel(c_id).await
            .expect("message(): unable to get the channel!")
            .guild().expect("message(): unable to get the guild channel!");
        
        let link = msg.link();
        
        //get the channel name to format it as: `#channel_name` in the embed
        let guild_channel = msg.channel(&ctx).await
            .expect("message(): unable to get the channel from the message!")
            .guild().expect("message(): unable to get the guild from the channel!");
        let channel_name = "#".to_owned() + guild_channel.name();
        let time = msg.timestamp;
        let display_color = color_hash(&channel_name, &author.tag(), time);
        let author_icon_url = author.face();

        log_channel.send_message(
            &ctx, 
            |reply| {
                reply.add_embed(|e| {
                    e.title(author.tag());
                    e.url(link);
                    e.description(channel_name);
                    e.field("posted:", msg.content, false);
                    e.timestamp(time);
                    e.color(Color::new(display_color));
                    e.thumbnail(author_icon_url)
                })
            })
        .await
        .unwrap();
    }

    //when a message is updated
    async fn message_update(&self, ctx: Context, updated: MessageUpdateEvent) {
        
        let author = match updated.author {
            Some(user) => user,
            None => return
        };

        //ignore messages from MessageLogger
        if author.bot && author.tag().as_str() == LOGGER_TAG {
            return;
        }

        let g_id = updated.guild_id
            .expect("message_update(): unable to get the guild_id!")
            .to_string();
        
        let save_map = read_json()
            .expect("message_update(): unable to read from json!");
        
        //ignore messages if logging not set up
        if !save_map.contains_key(&g_id) {
            return;
        }

        //get the channel id associated with the guild id
        let c_id = *save_map.get(&g_id)
            .expect("message_update(): unable to get the channel id from the hash map!");
        //turn the c_id into a guild channel
        let log_channel = ctx.http.get_channel(c_id).await
            .expect("message_update(): unable to get the channel!")
            .guild().expect("message_update(): unable to get the guild channel!");
        
        //change c_id to be the updated message's channel id to link to message
        //instead of the first log of the message
        let link = "https://discord.com/channels/".to_owned() 
            + &g_id + "/" + c_id.to_string().as_str() 
            + "/" + updated.id.to_string().as_str();
        
        //get the channel name to format it as: `#channel_name` in the embed
        let channels = updated.guild_id   //.channel(&ctx).await
            .expect("message_update(): unable to get the guild from the message!")
            .channels(&ctx).await.expect("message_update(): unable to get the channel from the guild");
        let guild_channel = channels.get(&updated.channel_id)
            .expect("message_update(): unable to get guild from channel!");
        let channel_name = "#".to_owned() + guild_channel.name();
        let updated_text = updated.content
            .expect("message_update(): unable to get the updated message!");
        let time = updated.timestamp
            .expect("message_update(): unable to get timestamp of original message!");
        let display_color = color_hash(&channel_name, &author.tag(), time);
        let edited_time = updated.edited_timestamp
            .expect("message_update(): unable to get timestamp of edited message!");
        let author_icon_url = author.face();

        log_channel.send_message(
            &ctx, 
            |reply| {
                reply.add_embed(|e| {
                    e.title(author.tag());
                    e.url(link);
                    e.description(channel_name);
                    e.field("edited:", updated_text, false);
                    e.timestamp(edited_time);
                    e.color(Color::new(display_color));
                    e.thumbnail(author_icon_url)
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
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MEMBERS;

    //build the client
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("error creating client!"); 

    if let Err(why) = client.start().await {
        println!("an error occurred while running the client: {:?}", why);
    }
}

fn read_json() -> Result<HashMap<String, u64>, std::io::Error> {
    //read from json file
    let contents = fs::read_to_string(JSON_PATH)?;

    let map: Result<SaveMap, Error>;
    //if the json file is empty, initialize the hash map
    //deserialize existing json file otherwise
    if contents == "" {
        let save_map = SaveMap{ map: HashMap::new() };
        map = Ok(save_map);
    } else {
        map = serde_json::from_str::<SaveMap>(&contents);
    }
    Ok(map.unwrap().map)
}

fn write_json(save_map: &HashMap<String, u64>) -> Result<(), std::io::Error> {
    //serialize and write to the json file
    let serialized = serde_json::to_string(&save_map)
        .expect("write_json(): unable to serialize the save_map!");
    fs::write(JSON_PATH, &serialized)?;
    Ok(())
}

//deletes entries in the json file if MessageLogger isn't in the server
fn sync_json(guild_list: Vec<UnavailableGuild>) -> Result<(), std::io::Error> {
    let mut map = read_json()?;
    map.iter_mut().for_each(|pair| {
        let mut exists = false;
        //iterate through guild list
        guild_list.iter().for_each(|guild| {
            if !exists && &guild.id.0.to_string() == pair.0 {
                exists = true;
            }
        });
        //delete entry if not found in the guild list
        if !exists {
            delete_entry(&pair.0);
        }
    });

    Ok(())
}

//turn (#channel, @user, timestamp) into a color
fn color_hash(channel_name: &String, user: &String, time: Timestamp) -> u32 {
    const TIMESTAMP_WEIGHT: u32 = 100;

    let mut hasher = DefaultHasher::new();
    channel_name.hash(&mut hasher);
    user.hash(&mut hasher);

    //get the number of seconds since the beginning of the day
    const SECONDS_IN_DAY: i64 = 86400;
    let beginning_of_day = time.unix_timestamp() - (time.unix_timestamp() % SECONDS_IN_DAY);
    let secs_of_day = time.unix_timestamp() - beginning_of_day;
    const MASK_TO_U32: i64 = 0x00000000FFFFFFFF;
    let secs_of_day: u32 = (secs_of_day & MASK_TO_U32).try_into()
        .expect("color_hash(): unable to convert masked secs_of_day to u32");

    //hash so the color value increases linearly with the timestamp
    let hashed_val = (hasher.finish() as u32) + (TIMESTAMP_WEIGHT * secs_of_day);
    
    //get it as a u24 (get rid of first 8 bits)
    let result = hashed_val & 0x00FFFFFF;
    result
}

//deletes the guild_id-channel_id pair from the json file if guild_id exists
fn delete_entry(g_id: &String) -> Option<u64> {
    let mut map = read_json().unwrap();
    //remove the entry from the map if it exists
    let return_val = map.remove(g_id);
    //update the json file
    write_json(&map).unwrap();
    return_val
}

