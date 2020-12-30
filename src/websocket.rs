use crate::gamecoordinator::GameCoordinator;
use crate::gamestate::PlayerID;
use rmp_serde;
use std::{
    collections::HashMap,
    io::{self, Cursor},
    net::ToSocketAddrs,
    sync::{mpsc::Sender, Arc, Mutex, RwLock},
    thread,
};
use websocket::sync as ws;
use websocket::{
    result::WebSocketError,
    server::{InvalidConnection, NoTlsAcceptor},
    sync::{server::upgrade::Buffer, stream::TcpStream},
    Message, OwnedMessage,
};

// complicated stack to share the error_channel among the client threads
type ErrorChannel = Option<Arc<Mutex<Sender<GameServerError>>>>;

pub struct GameServer {
    ws: ws::Server<NoTlsAcceptor>,
    coordinator: Arc<RwLock<GameCoordinator>>,
    error_channel: ErrorChannel,
    clients: Arc<RwLock<HashMap<PlayerID, Arc<Mutex<ws::Client<TcpStream>>>>>>,
}

#[derive(Debug)]
pub enum GameServerError {
    WebSocketError(WebSocketError),
    BadMessageType(PlayerID, OwnedMessage),
    SerializeError(PlayerID),
    IOError(io::Error),
    OtherError(String),
}

impl From<io::Error> for GameServerError {
    fn from(err: io::Error) -> GameServerError {
        GameServerError::IOError(err)
    }
}

impl From<InvalidConnection<TcpStream, Buffer>> for GameServerError {
    fn from(err: InvalidConnection<TcpStream, Buffer>) -> GameServerError {
        //TODO: Improve this error conversion
        GameServerError::OtherError(format!("{:?}", err))
    }
}

impl From<WebSocketError> for GameServerError {
    fn from(err: WebSocketError) -> GameServerError {
        GameServerError::WebSocketError(err)
    }
}

impl GameServer {
    pub fn new(
        addr: impl ToSocketAddrs,
        error_channel: Option<Sender<GameServerError>>,
        game_coordinator: GameCoordinator,
    ) -> Result<Self, GameServerError> {
        Ok(GameServer {
            ws: ws::Server::bind(addr)?,
            coordinator: Arc::new(RwLock::new(game_coordinator)),
            error_channel: error_channel.map(|e| Arc::new(Mutex::new(e))),
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn send_error(error_channel: &ErrorChannel, e: impl Into<GameServerError>) {
        if let Some(channel) = error_channel {
            channel.lock().unwrap().send(e.into()).unwrap();
        }
    }

    pub fn run(&mut self) {
        loop {
            let connection = self.ws.accept();
            match connection {
                Ok(ws_upgrade) => match ws_upgrade.accept() {
                    Ok(client) => {
                        let player_id = self.coordinator.write().unwrap().on_new_user();
                        let client = Arc::new(Mutex::new(client));
                        self.clients
                            .write()
                            .unwrap()
                            .insert(player_id, client.clone());

                        let coordinator = self.coordinator.clone();
                        let error_channel = self.error_channel.clone();
                        let hash_map_ref = self.clients.clone();
                        thread::spawn(move || {
                            Self::handle_client(
                                client,
                                player_id,
                                coordinator.clone(),
                                error_channel,
                            );
                            coordinator.write().unwrap().on_dropped_user(player_id);
                            hash_map_ref.write().unwrap().remove(&player_id);
                        });
                    }
                    Err((_stream, e)) => Self::send_error(&self.error_channel, e),
                },
                Err(e) => Self::send_error(&self.error_channel, e),
            }
        }
    }

    fn handle_client(
        client: Arc<Mutex<ws::Client<TcpStream>>>,
        player_id: PlayerID,
        coordinator: Arc<RwLock<GameCoordinator>>,
        error_channel: ErrorChannel,
    ) {
        // TODO: some of these unwraps should be replaced with a return to avoid
        // posioning the entire server
        client.lock().unwrap().set_nonblocking(true).unwrap();
        loop {
            match client.lock().unwrap().recv_message() {
                Err(WebSocketError::NoDataAvailable) => {}
                Err(e) => {
                    Self::send_error(&error_channel, e);
                    return;
                }
                Ok(msg) => match msg {
                    OwnedMessage::Binary(data) => {
                        let action = match rmp_serde::from_read(Cursor::new(data)) {
                            Ok(action) => action,
                            Err(_e) => {
                                Self::send_error(
                                    &error_channel,
                                    GameServerError::SerializeError(player_id),
                                );
                                return;
                            }
                        };
                        let events = coordinator
                            .write()
                            .unwrap()
                            .handle_action(player_id, action);
                        if events.len() >= 1 {
                            let data = rmp_serde::to_vec(&events).unwrap();
                            match client.lock().unwrap().send_message(&Message::binary(data)) {
                                Ok(()) => {}
                                Err(e) => {
                                    Self::send_error(&error_channel, e);
                                    return;
                                }
                            };
                        }
                    }
                    _ => Self::send_error(
                        &error_channel,
                        GameServerError::BadMessageType(player_id, msg),
                    ),
                },
            }
        }
    }
}
