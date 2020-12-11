use plugin_api::{*,events::*};

#[export_function]
pub fn on_load(load: PluginLoadEvent) -> () {
    send_actions(vec![Action::Print("This is a test".to_owned())]);
    println!("Hello world");
    
}

#[export_function]
pub fn on_player_join(input: PlayerJoinEvent) -> PlayerJoinResult {
    send_actions(vec![Action::PlayerSendMessage(input.player_id,format!("Welcome {} on our server",input.player_name))]);
    if input.player_name == "Cheater123" {
        PlayerJoinResult::CloseConnection
    } else {
        PlayerJoinResult::None
    }
}
