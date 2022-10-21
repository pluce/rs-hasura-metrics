use env_logger::{Builder, Env};
use log::info;
use prometheus_exporter::prometheus::register_int_gauge_vec;
use std::env;
use std::net::SocketAddr;
use tokio_postgres::{Client, Config, Error, NoTls};

async fn make_client(config: &mut Config) -> Result<Client, Error> {
    // Connect to the database.
    let (client, connection) = config.connect(NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    return Ok(client);
}
#[derive(Debug)]
struct EventInvocationCountStatus {
    event: String,
    status: Option<i32>,
    count: i64,
}
#[derive(Debug)]
struct EventCountDelivered {
    event: String,
    count: i64,
    delivered: bool,
}
#[derive(Debug)]
struct Stats {
    count_invocations: Vec<EventInvocationCountStatus>,
    count_events: Vec<EventCountDelivered>,
}

async fn get_stats(client: &Client) -> Result<Stats, Error> {
    // Now we can execute a simple statement that just returns its parameter.
    let rows_invocation = client
        .query(
            "SELECT trigger_name as event, status, COUNT(distinct i.id) as count FROM \"hdb_catalog\".\"event_invocation_logs\" i
	INNER JOIN \"hdb_catalog\".\"event_log\" e ON i.event_id = e.id
	GROUP BY trigger_name, status;",
            &[],
        );

    // Now we can execute a simple statement that just returns its parameter.
    let rows_events = client
        .query(
            "SELECT trigger_name as event, delivered, COUNT(distinct e.id) as count FROM \"hdb_catalog\".\"event_log\" e GROUP BY trigger_name, delivered;",
            &[],
        );

    // And then check that we got back the same string we sent over.
    let result_invocations = rows_invocation
        .await?
        .into_iter()
        .map(|x| EventInvocationCountStatus {
            event: x.get(0),
            status: x.get(1),
            count: x.get(2),
        })
        .collect(); //rows[0].get(0);
    println!("{:?}", result_invocations);

    // And then check that we got back the same string we sent over.
    let result_events = rows_events
        .await?
        .into_iter()
        .map(|x| EventCountDelivered {
            event: x.get(0),
            count: x.get(2),
            delivered: x.get(1),
        })
        .collect(); //rows[0].get(0);

    println!("{:?}", result_events);

    Ok(Stats {
        count_invocations: result_invocations,
        count_events: result_events,
    })
}

#[tokio::main]
async fn main() {
    // Setup logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse address used to bind exporter to.
    let addr_raw = format!(
        "{}:{}",
        &env::var("LISTEN").unwrap_or("127.0.0.1".to_string()),
        &env::var("PORT").unwrap_or("9185".to_string())
    );
    let addr: SocketAddr = addr_raw.parse().expect("can not parse listen addr");

    let gauge_invocations = register_int_gauge_vec!(
        "count_invocations",
        "event invocations count",
        &["event", "status"]
    )
    .expect("can not create gauge count_invocations");

    let gauge_events =
        register_int_gauge_vec!("count_events", "event count", &["event", "delivered"])
            .expect("can not create gauge count_events");

    let mut builder = prometheus_exporter::Builder::new(addr);

    builder
        .with_endpoint("/metrics")
        .expect("failed to set endpoint");

    let exporter = builder.start().expect("can not start exporter");

    let mut config = Config::new();
    config
        .user(&env::var("POSTGRES_DB_USER").unwrap_or("postgres".to_string()))
        .password(&env::var("POSTGRES_DB_PASSWORD").unwrap_or("".to_string()))
        .host(&env::var("POSTGRES_DB_HOST").unwrap_or("localhost".to_string()))
        .port(
            env::var("POSTGRES_DB_PORT")
                .unwrap_or("5432".to_string())
                .to_string()
                .parse::<u16>()
                .unwrap(),
        )
        .dbname(&env::var("POSTGRES_DB_METADATA").unwrap_or("postgres".to_string()));

    let client = make_client(&mut config).await.unwrap();
    loop {
        // Will block until a new request comes in.
        let _guard = exporter.wait_request();
        info!("Updating metrics");
        let result = get_stats(&client).await.unwrap();
        result.count_invocations.into_iter().for_each(|v| {
            gauge_invocations
                .with_label_values(&[&v.event, &v.status.unwrap_or(-1).to_string()])
                .set(v.count)
        });
        result.count_events.into_iter().for_each(|v| {
            gauge_events
                .with_label_values(&[&v.event, &v.delivered.to_string()])
                .set(v.count)
        });
    }
}
