use std::sync::mpsc;
use futures_util::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use tauri::Manager;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

// 通信のはじめ
// write: 送信用の関数
// msg: メッセージ
async fn on_open(
    write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    msg: &str
) {
    let request_msg = Message::Text(format!("{}", msg));
    write.send(request_msg).await.expect("Failed to send registration message");
}

// メッセージの送受信
// 接続したwebsocketのやり取りを行う
// read: 受信用の関数
// write: 送信用の関数
async fn on_message(
    mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
    receiver: mpsc::Sender<Message>
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                println!("Received a message: {}", msg);
                receiver.send(msg).unwrap();
            },
            Err(e) => {
                eprintln!("Error reading message: {:?}", e);
            }
        }
    }
}

async fn client(
    app: tauri::AppHandle,
    outside_receiver: mpsc::Receiver<Message>
) {
    let url = "wss://echo.websocket.events";

    // URLのセッティング
    println!("Connecting to - {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to Agent Network");

    // 受信と送信用関数を切り離し
    let (mut write, read) = ws_stream.split();

    // wss://echo.websocket.eventsには必要ないが、オブジェクトの描き方がてら、適当な文字を入れておく。
    let start_msg = r#"{{
        "context": "message",
        "object_tmp": {{
            "hoge": "hoge"
        }}
    }}"#;
    // websocketの接続
    // register the weboscoket
    on_open(&mut write, start_msg).await;
    

    // 関数(outside)からの値の監視
    let (inside_sender, inside_recevier) = mpsc::channel::<Message>();

    // メッセージの受信用にスレッドを開始
    // Handle incoming messages in a separate task
    let on_message = tokio::spawn(async move {
        on_message(read, inside_sender).await;
    });

    // websocketから受け取った値をReactへ送信する
    let send_outside = tokio::spawn(async move {
        loop{
            let msg = inside_recevier.recv().unwrap();
            app.emit_all("back-to-front", msg.to_string()).unwrap();
        }
    });

    // メッセージの送信
    let send_mesager = tokio::spawn(async move {
        loop{
            let msg = outside_receiver.recv().unwrap();
            println!("SendMessage {}", msg);
            write.send(msg).await.expect("Failed to send registration message");
        }
    });

    // tokioにスレッドをぶん投げ
    // Await both tasks (optional, depending on your use case)
    let _ = tokio::try_join!(on_message, send_outside, send_mesager);
}

pub fn websocket_setup(app: &mut tauri::App, receiver: mpsc::Receiver<Message>) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(client(app_handle, receiver));
    });
    Ok(())
}
