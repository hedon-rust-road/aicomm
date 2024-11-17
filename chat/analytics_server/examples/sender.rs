use analytics_server::pb::{
    analytics_event::EventType, app_exit_event::ExitCode, AnalyticsEvent, AppExitEvent,
    EventContext, GeoLocation, SystemInfo,
};
use anyhow::Result;
use prost::Message;

#[tokio::main]
async fn main() -> Result<()> {
    let mut context = EventContext {
        client_id: "client_123".to_string(),
        user_id: "user_123".to_string(),
        app_version: "1.0.0".to_string(),
        client_ts: chrono::Utc::now().timestamp_millis(),
        ..Default::default()
    };
    // this should be overwritten by server
    context.server_ts = chrono::Utc::now().timestamp_millis();
    context.user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36".to_string();
    context.ip = "127.0.0.1".to_string();
    context.system = Some(SystemInfo {
        os: "macos".to_string(),
        arch: "x64".to_string(),
        locale: "en-US".to_string(),
        timezone: "Asia/Shanghai".to_string(),
    });
    // this should be overwritten by server
    context.geo = Some(GeoLocation {
        country: "China".to_string(),
        region: "Shanghai".to_string(),
        city: "Shanghai".to_string(),
    });

    let exit = AppExitEvent {
        exit_code: ExitCode::Success.into(),
    };
    let event = AnalyticsEvent {
        context: Some(context),
        event_type: Some(EventType::AppExit(exit)),
    };
    println!("{:?}", event);

    let data = Message::encode_to_vec(&event);
    // write data to "../fixtures/event.bin"
    std::fs::create_dir_all("../fixtures")?;
    std::fs::write("../fixtures/event.bin", data)?;
    // load data from "../fixtures/event.bin"
    let data = std::fs::read("../fixtures/event.bin")?;
    let event = AnalyticsEvent::decode(data.as_slice())?;
    println!("{:?}", event);

    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:6690/api/event")
        .header("content-type", "application/protobuf")
        .body(data)
        .send()
        .await?;

    println!("server returned: {:?}", res);
    let body = res.text().await?;
    println!("server returned body: {}", body);
    Ok(())
}
