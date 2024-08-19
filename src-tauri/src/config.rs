use tokio_tungstenite::tungstenite::Message;
use std::sync::mpsc;

#[derive(Debug)]
pub struct AppState {
    pub sender:  mpsc::Sender<Message>,
}

// websocketへの値を共有するための関数で使うState
impl AppState {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            sender: sender,
        }
    }
}