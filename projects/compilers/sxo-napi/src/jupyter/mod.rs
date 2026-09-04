//! Pure-Rust Jupyter kernel (ZeroMQ via `zeromq`, no libzmq).
//!
//! Hosted inside `sxo-napi` for native Node launches only.

mod connection;
mod wire;

use crate::session::Session;
use connection::ConnectionFile;
use serde_json::{Value, json};
use sxo_types::VERSION as CORE_VERSION;
use wire::JupyterMessage;
use zeromq::{PubSocket, RepSocket, RouterSocket, Socket, SocketRecv, SocketSend};

/// Run a Jupyter kernel until `shutdown_request` (blocks the async runtime).
pub async fn run(connection_file: &str) -> Result<(), String> {
    let conn = ConnectionFile::load(connection_file)?;
    let key = conn.key.clone();

    let mut shell = RouterSocket::new();
    shell.bind(&conn.endpoint(conn.shell_port)).await.map_err(|e| format!("bind shell: {e}"))?;

    let mut control = RouterSocket::new();
    control.bind(&conn.endpoint(conn.control_port)).await.map_err(|e| format!("bind control: {e}"))?;

    let mut iopub = PubSocket::new();
    iopub.bind(&conn.endpoint(conn.iopub_port)).await.map_err(|e| format!("bind iopub: {e}"))?;

    let mut stdin = RouterSocket::new();
    stdin.bind(&conn.endpoint(conn.stdin_port)).await.map_err(|e| format!("bind stdin: {e}"))?;

    let mut hb = RepSocket::new();
    hb.bind(&conn.endpoint(conn.hb_port)).await.map_err(|e| format!("bind hb: {e}"))?;

    // Drain stdin quietly; Jupyter may open the channel without using it.
    tokio::spawn(async move {
        loop {
            if stdin.recv().await.is_err() {
                break;
            }
        }
    });

    // Heartbeat: echo whatever arrives.
    tokio::spawn(async move {
        loop {
            match hb.recv().await {
                Ok(msg) => {
                    if hb.send(msg).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let eng = Session::new();
    let mut execution_count: i64 = 0;
    let mut shutting_down = false;

    while !shutting_down {
        tokio::select! {
            msg = shell.recv() => {
                let msg = msg.map_err(|e| format!("shell recv: {e}"))?;
                let request = JupyterMessage::from_zmq(msg, &key)?;
                shutting_down = handle_request(
                    &request,
                    &key,
                    &eng,
                    &mut execution_count,
                    &mut shell,
                    &mut iopub,
                    Channel::Shell,
                ).await?;
            }
            msg = control.recv() => {
                let msg = msg.map_err(|e| format!("control recv: {e}"))?;
                let request = JupyterMessage::from_zmq(msg, &key)?;
                shutting_down = handle_request(
                    &request,
                    &key,
                    &eng,
                    &mut execution_count,
                    &mut control,
                    &mut iopub,
                    Channel::Control,
                ).await?;
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum Channel {
    Shell,
    Control,
}

async fn handle_request(
    request: &JupyterMessage,
    key: &str,
    eng: &Session,
    execution_count: &mut i64,
    reply_sock: &mut RouterSocket,
    iopub: &mut PubSocket,
    channel: Channel,
) -> Result<bool, String> {
    match request.msg_type() {
        "kernel_info_request" => {
            let content = kernel_info_content();
            send_reply(reply_sock, key, &request.reply("kernel_info_reply", content)).await?;
            Ok(false)
        }
        "comm_info_request" => {
            let content = json!({ "comms": {}, "status": "ok" });
            send_reply(reply_sock, key, &request.reply("comm_info_reply", content)).await?;
            Ok(false)
        }
        "shutdown_request" => {
            let restart = request.content.get("restart").and_then(|v| v.as_bool()).unwrap_or(false);
            let content = json!({ "status": "ok", "restart": restart });
            send_reply(reply_sock, key, &request.reply("shutdown_reply", content)).await?;
            Ok(true)
        }
        "interrupt_request" => {
            let content = json!({ "status": "ok" });
            send_reply(reply_sock, key, &request.reply("interrupt_reply", content)).await?;
            Ok(false)
        }
        "execute_request" if matches!(channel, Channel::Shell) => {
            handle_execute(request, key, eng, execution_count, reply_sock, iopub).await?;
            Ok(false)
        }
        "is_complete_request" => {
            let content = json!({ "status": "unknown" });
            send_reply(reply_sock, key, &request.reply("is_complete_reply", content)).await?;
            Ok(false)
        }
        _ => {
            // Silently ignore unknown / unsupported messages.
            Ok(false)
        }
    }
}

async fn handle_execute(
    request: &JupyterMessage,
    key: &str,
    eng: &Session,
    execution_count: &mut i64,
    shell: &mut RouterSocket,
    iopub: &mut PubSocket,
) -> Result<(), String> {
    let code = request.content.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let silent = request.content.get("silent").and_then(|v| v.as_bool()).unwrap_or(false);
    let store_history = request.content.get("store_history").and_then(|v| v.as_bool()).unwrap_or(true);

    if !silent && store_history {
        *execution_count += 1;
    }
    let count = *execution_count;

    publish(iopub, key, &request.iopub("status", "status", json!({ "execution_state": "busy" }))).await?;

    if !silent {
        publish(
            iopub,
            key,
            &request.iopub("execute_input", "execute_input", json!({ "code": code, "execution_count": count })),
        )
        .await?;
    }

    match evaluate_mathematica(eng, code) {
        Ok(EvalOut::Text(text)) => {
            if !silent {
                publish(
                    iopub,
                    key,
                    &request.iopub(
                        "execute_result",
                        "execute_result",
                        json!({
                            "execution_count": count,
                            "data": { "text/plain": text },
                            "metadata": {},
                        }),
                    ),
                )
                .await?;
            }
            send_reply(
                shell,
                key,
                &request.reply(
                    "execute_reply",
                    json!({
                        "status": "ok",
                        "execution_count": count,
                        "user_expressions": {},
                        "payload": [],
                    }),
                ),
            )
            .await?;
        }
        Ok(EvalOut::Svg { svg, plain }) => {
            if !silent {
                publish(
                    iopub,
                    key,
                    &request.iopub(
                        "display_data",
                        "display_data",
                        json!({
                            "data": {
                                "image/svg+xml": svg,
                                "text/plain": plain,
                            },
                            "metadata": {},
                        }),
                    ),
                )
                .await?;
            }
            send_reply(
                shell,
                key,
                &request.reply(
                    "execute_reply",
                    json!({
                        "status": "ok",
                        "execution_count": count,
                        "user_expressions": {},
                        "payload": [],
                    }),
                ),
            )
            .await?;
        }
        Err(err) => {
            publish(
                iopub,
                key,
                &request.iopub(
                    "error",
                    "error",
                    json!({
                        "ename": "SxoError",
                        "evalue": err,
                        "traceback": [err],
                    }),
                ),
            )
            .await?;
            send_reply(
                shell,
                key,
                &request.reply(
                    "execute_reply",
                    json!({
                        "status": "error",
                        "execution_count": count,
                        "ename": "SxoError",
                        "evalue": err,
                        "traceback": [err],
                    }),
                ),
            )
            .await?;
        }
    }

    publish(iopub, key, &request.iopub("status", "status", json!({ "execution_state": "idle" }))).await?;
    Ok(())
}

enum EvalOut {
    Text(String),
    Svg { svg: String, plain: String },
}

fn evaluate_mathematica(eng: &Session, code: &str) -> Result<EvalOut, String> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Ok(EvalOut::Text(String::new()));
    }
    let w = eng.parse_mathematica(trimmed).map_err(|e| e.message.clone())?;
    let term = eng.lower_mathematica(&w);
    if let Some(plot) = eng.try_plot_svg(term, sxo_types::Dialect::Mathematica) {
        let svg = plot.map_err(|e| e.message.clone())?;
        let plain = eng.render_as_wolfram(term);
        return Ok(EvalOut::Svg { svg, plain });
    }
    let evaluated = eng.evaluate(term);
    let simplified = eng.simplify_term(evaluated);
    Ok(EvalOut::Text(eng.render_as_wolfram(simplified)))
}

fn kernel_info_content() -> Value {
    json!({
        "status": "ok",
        "protocol_version": "5.3",
        "implementation": "SXO Mathematica",
        "implementation_version": CORE_VERSION,
        "language_info": {
            "name": "wolfram",
            "version": CORE_VERSION,
            "mimetype": "text/x-wolfram",
            "file_extension": ".wl",
            "pygments_lexer": "mathematica",
            "codemirror_mode": "mathematica",
        },
        "banner": "SXO Mathematica — local Mathematica Form via SXO",
        "help_links": [],
    })
}

async fn send_reply(sock: &mut RouterSocket, key: &str, msg: &JupyterMessage) -> Result<(), String> {
    let zmq = msg.to_zmq(key)?;
    sock.send(zmq).await.map_err(|e| format!("send reply: {e}"))
}

async fn publish(iopub: &mut PubSocket, key: &str, msg: &JupyterMessage) -> Result<(), String> {
    let zmq = msg.to_zmq(key)?;
    iopub.send(zmq).await.map_err(|e| format!("iopub send: {e}"))
}
