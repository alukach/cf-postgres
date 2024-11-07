use std::str::FromStr;
use tokio_postgres::{config::Host, Config};
use worker::{
    console_log, event, postgres_tls::PassthroughTls, wasm_bindgen_futures, Context, Env, Request,
    Response, SecureTransport, Socket,
};

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> anyhow::Result<Response> {
    // Get connection details
    let db_connection_str = env.secret("DB_CONN_STR")?;
    let config = Config::from_str(&db_connection_str.to_string())?;
    let Host::Tcp(host) = &config.get_hosts()[0] else {
        return Err(anyhow::anyhow!("No host found"));
    };
    let port = config.get_ports()[0];

    // Connect using Worker Socket
    console_log!("Connecting to database at {}:{}", host, port);
    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect(host, port)?;
    let (client, connection) = config.connect_raw(socket, PassthroughTls).await?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    // Query database
    console_log!("Getting time from DB");
    let result = client.query("SELECT NOW()::text;", &[]).await?;
    let time = result[0].get::<_, String>(0);

    Ok(Response::ok(format!("DB time: {}", time))?)
}
