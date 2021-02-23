mod client_init;
mod scene;
mod ui;

use super::char_selection::CharSelectionState;
#[cfg(feature = "singleplayer")]
use crate::singleplayer::Singleplayer;
use crate::{
    i18n::{i18n_asset_key, Localization},
    render::Renderer,
    settings::Settings,
    window::Event,
    Direction, GlobalState, PlayState, PlayStateResult,
};
use client::{
    addr::ConnectionArgs,
    error::{InitProtocolError, NetworkConnectError, NetworkError},
};
use client_init::{ClientConnArgs, ClientInit, Error as InitError, Msg as InitMsg};
use common::{assets::AssetExt, comp, span};
use scene::Scene;
use std::sync::Arc;
use tokio::runtime;
use tracing::error;
use ui::{Event as MainMenuEvent, MainMenuUi};

pub struct MainMenuState {
    main_menu_ui: MainMenuUi,
    // Used for client creation.
    client_init: Option<ClientInit>,
    scene: Scene,
}

impl MainMenuState {
    /// Create a new `MainMenuState`.
    pub fn new(global_state: &mut GlobalState) -> Self {
        Self {
            main_menu_ui: MainMenuUi::new(global_state),
            client_init: None,
            scene: Scene::new(global_state.window.renderer_mut()),
        }
    }
}

impl PlayState for MainMenuState {
    fn enter(&mut self, global_state: &mut GlobalState, _: Direction) {
        // Kick off title music
        if global_state.settings.audio.output.is_enabled() && global_state.audio.music_enabled() {
            global_state.audio.play_title_music();
        }

        // Reset singleplayer server if it was running already
        #[cfg(feature = "singleplayer")]
        {
            global_state.singleplayer = None;
        }

        // Updated localization in case the selected language was changed
        self.main_menu_ui
            .update_language(global_state.i18n, &global_state.settings);
        // Set scale mode in case it was change
        self.main_menu_ui
            .set_scale_mode(global_state.settings.gameplay.ui_scale);
    }

    #[allow(clippy::single_match)] // TODO: remove when event match has multiple arms
    fn tick(&mut self, global_state: &mut GlobalState, events: Vec<Event>) -> PlayStateResult {
        span!(_guard, "tick", "<MainMenuState as PlayState>::tick");

        // Poll server creation
        #[cfg(feature = "singleplayer")]
        {
            if let Some(singleplayer) = &global_state.singleplayer {
                match singleplayer.receiver.try_recv() {
                    Ok(Ok(runtime)) => {
                        let server_settings = singleplayer.settings();
                        // Attempt login after the server is finished initializing
                        attempt_login(
                            &mut global_state.settings,
                            &mut global_state.info_message,
                            "singleplayer".to_owned(),
                            "".to_owned(),
                            ClientConnArgs::Resolved(ConnectionArgs::IpAndPort(vec![
                                server_settings.gameserver_address,
                            ])),
                            &mut self.client_init,
                            Some(runtime),
                        );
                    },
                    Ok(Err(e)) => {
                        error!(?e, "Could not start server");
                        global_state.singleplayer = None;
                        self.client_init = None;
                        self.main_menu_ui.cancel_connection();
                        self.main_menu_ui.show_info(format!("Error: {:?}", e));
                    },
                    Err(_) => (),
                }
            }
        }

        // Handle window events.
        for event in events {
            // Pass all events to the ui first.
            if self.main_menu_ui.handle_event(event.clone()) {
                continue;
            }

            match event {
                Event::Close => return PlayStateResult::Shutdown,
                // Ignore all other events.
                _ => {},
            }
        }
        // Poll client creation.
        match self.client_init.as_ref().and_then(|init| init.poll()) {
            Some(InitMsg::Done(Ok(mut client))) => {
                self.client_init = None;
                self.main_menu_ui.connected();
                // Register voxygen components / resources
                crate::ecs::init(client.state_mut().ecs_mut());
                return PlayStateResult::Push(Box::new(CharSelectionState::new(
                    global_state,
                    std::rc::Rc::new(std::cell::RefCell::new(client)),
                )));
            },
            Some(InitMsg::Done(Err(err))) => {
                let localized_strings = global_state.i18n.read();
                self.client_init = None;
                global_state.info_message = Some({
                    let err = match err {
                        InitError::NoAddress => {
                            localized_strings.get("main.login.server_not_found").into()
                        },
                        InitError::ClientError(err) => match err {
                            client::Error::AuthErr(e) => format!(
                                "{}: {}",
                                localized_strings.get("main.login.authentication_error"),
                                e
                            ),
                            client::Error::TooManyPlayers => {
                                localized_strings.get("main.login.server_full").into()
                            },
                            client::Error::AuthServerNotTrusted => localized_strings
                                .get("main.login.untrusted_auth_server")
                                .into(),
                            client::Error::ServerWentMad => localized_strings
                                .get("main.login.outdated_client_or_server")
                                .into(),
                            client::Error::ServerTimeout => {
                                localized_strings.get("main.login.timeout").into()
                            },
                            client::Error::ServerShutdown => {
                                localized_strings.get("main.login.server_shut_down").into()
                            },
                            client::Error::AlreadyLoggedIn => {
                                localized_strings.get("main.login.already_logged_in").into()
                            },
                            client::Error::NotOnWhitelist => {
                                localized_strings.get("main.login.not_on_whitelist").into()
                            },
                            client::Error::Banned(reason) => format!(
                                "{}: {}",
                                localized_strings.get("main.login.banned"),
                                reason
                            ),
                            client::Error::InvalidCharacter => {
                                localized_strings.get("main.login.invalid_character").into()
                            },
                            client::Error::NetworkErr(NetworkError::ConnectFailed(
                                NetworkConnectError::Handshake(InitProtocolError::WrongVersion(_)),
                            )) => localized_strings
                                .get("main.login.network_wrong_version")
                                .into(),
                            client::Error::NetworkErr(e) => format!(
                                "{}: {:?}",
                                localized_strings.get("main.login.network_error"),
                                e
                            ),
                            client::Error::ParticipantErr(e) => format!(
                                "{}: {:?}",
                                localized_strings.get("main.login.network_error"),
                                e
                            ),
                            client::Error::StreamErr(e) => format!(
                                "{}: {:?}",
                                localized_strings.get("main.login.network_error"),
                                e
                            ),
                            client::Error::Other(e) => {
                                format!("{}: {}", localized_strings.get("common.error"), e)
                            },
                            client::Error::AuthClientError(e) => match e {
                                client::AuthClientError::InvalidUrl(e) => format!(
                                    "{}: {}",
                                    localized_strings.get("common.fatal_error"),
                                    e
                                ),
                                // TODO: remove parentheses
                                client::AuthClientError::RequestError(e) => format!(
                                    "{}: {}",
                                    localized_strings.get("main.login.failed_sending_request"),
                                    e
                                ),
                                client::AuthClientError::ServerError(_, e) => e,
                            },
                        },
                        InitError::ClientCrashed => {
                            localized_strings.get("main.login.client_crashed").into()
                        },
                    };
                    // Log error for possible additional use later or incase that the error
                    // displayed is cut of.
                    error!("{}", err);
                    err
                });
            },
            Some(InitMsg::IsAuthTrusted(auth_server)) => {
                if global_state
                    .settings
                    .networking
                    .trusted_auth_servers
                    .contains(&auth_server)
                {
                    // Can't fail since we just polled it, it must be Some
                    self.client_init
                        .as_ref()
                        .unwrap()
                        .auth_trust(auth_server, true);
                } else {
                    // Show warning that auth server is not trusted and prompt for approval
                    self.main_menu_ui.auth_trust_prompt(auth_server);
                }
            },
            None => {},
        }

        // Maintain the UI.
        for event in self
            .main_menu_ui
            .maintain(global_state, global_state.clock.dt())
        {
            match event {
                MainMenuEvent::LoginAttempt {
                    username,
                    password,
                    server_address,
                } => {
                    let mut net_settings = &mut global_state.settings.networking;
                    net_settings.username = username.clone();
                    net_settings.default_server = server_address.clone();
                    if !net_settings.servers.contains(&server_address) {
                        net_settings.servers.push(server_address.clone());
                    }
                    global_state.settings.save_to_file_warn();

                    attempt_login(
                        &mut global_state.settings,
                        &mut global_state.info_message,
                        username,
                        password,
                        ClientConnArgs::Host(server_address),
                        &mut self.client_init,
                        None,
                    );
                },
                MainMenuEvent::CancelLoginAttempt => {
                    // client_init contains Some(ClientInit), which spawns a thread which contains a
                    // TcpStream::connect() call This call is blocking
                    // TODO fix when the network rework happens
                    #[cfg(feature = "singleplayer")]
                    {
                        global_state.singleplayer = None;
                    }
                    self.client_init = None;
                    self.main_menu_ui.cancel_connection();
                },
                MainMenuEvent::ChangeLanguage(new_language) => {
                    global_state.settings.language.selected_language =
                        new_language.language_identifier;
                    global_state.i18n = Localization::load_expect(&i18n_asset_key(
                        &global_state.settings.language.selected_language,
                    ));
                    global_state.i18n.read().log_missing_entries();
                    self.main_menu_ui
                        .update_language(global_state.i18n, &global_state.settings);
                },
                #[cfg(feature = "singleplayer")]
                MainMenuEvent::StartSingleplayer => {
                    let singleplayer = Singleplayer::new();

                    global_state.singleplayer = Some(singleplayer);
                },
                MainMenuEvent::Quit => return PlayStateResult::Shutdown,
                // Note: Keeping in case we re-add the disclaimer
                /*MainMenuEvent::DisclaimerAccepted => {
                    global_state.settings.show_disclaimer = false
                },*/
                MainMenuEvent::AuthServerTrust(auth_server, trust) => {
                    if trust {
                        global_state
                            .settings
                            .networking
                            .trusted_auth_servers
                            .insert(auth_server.clone());
                        global_state.settings.save_to_file_warn();
                    }
                    self.client_init
                        .as_ref()
                        .map(|init| init.auth_trust(auth_server, trust));
                },
            }
        }

        if let Some(info) = global_state.info_message.take() {
            self.main_menu_ui.show_info(info);
        }

        PlayStateResult::Continue
    }

    fn name(&self) -> &'static str { "Title" }

    fn render(&mut self, renderer: &mut Renderer, _: &Settings) {
        // TODO: maybe the drawer should be passed in from above?
        let mut drawer = match renderer
            .start_recording_frame(self.scene.global_bind_group())
            .unwrap()
        {
            Some(d) => d,
            // Couldn't get swap chain texture this fime
            None => return,
        };

        // Draw the UI to the screen.
        self.main_menu_ui.render(&mut drawer.third_pass().draw_ui());
    }
}

fn attempt_login(
    settings: &mut Settings,
    info_message: &mut Option<String>,
    username: String,
    password: String,
    connection_args: ClientConnArgs,
    client_init: &mut Option<ClientInit>,
    runtime: Option<Arc<runtime::Runtime>>,
) {
    if comp::Player::alias_is_valid(&username) {
        // Don't try to connect if there is already a connection in progress.
        if client_init.is_none() {
            *client_init = Some(ClientInit::new(
                connection_args,
                username,
                Some(settings.graphics.view_distance),
                password,
                runtime,
            ));
        }
    } else {
        *info_message = Some("Invalid username".to_string());
    }
}
