use bsky_sdk::{
    api::{
        app::bsky::{
            embed::images::{ImageData, MainData},
            feed::post::{RecordData, RecordEmbedRefs},
        },
        types::{string::Datetime, Union},
    },
    BskyAgent,
};
use chrono::{DateTime, Duration, Timelike, Utc};
use chrono_tz::Europe::Helsinki;

use crate::{Aggregates, Title};

pub async fn post(
    bluesky_username: &String,
    bluesky_password: &String,
    image_filename: &str,
    aggregates: &Aggregates,
    title: &Title,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    agent.login(bluesky_username, bluesky_password).await?;

    let image_bytes = std::fs::read(image_filename)?;

    let output = agent.api.com.atproto.repo.upload_blob(image_bytes).await?;

    let image_data = ImageData {
        alt: "".to_string(),
        aspect_ratio: None,
        image: output.data.blob,
    };

    let images = vec![image_data.into()];

    let timerange = |ts: DateTime<Utc>| {
        let ts1 = ts.with_timezone(&Helsinki);
        let ts2 = ts1 + Duration::hours(1);
        format!("{:02}-{:02}", ts1.hour(), ts2.hour())
    };

    let text_header = format!(
        "Pörssisähkön spot-hinnat {}na {}",
        title.weekday, title.date
    );
    let text_avg = format!("Keskihinta: {:.2} c/kWh", aggregates.avg);
    let text_min = format!(
        "Minimi: {:.2} c/kWh (klo {})",
        aggregates.min.1,
        timerange(aggregates.min.0)
    );
    let text_max = format!(
        "Maksimi: {:.2} c/kWh (klo {})",
        aggregates.max.1,
        timerange(aggregates.max.0)
    );

    let text = format!(
        "{}\n\n{}\n{}\n{}",
        text_header, text_avg, text_min, text_max
    );

    let embed = Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(
        Box::new(MainData { images }.into()),
    )));

    agent
        .create_record(RecordData {
            created_at: Datetime::now(),
            embed,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text,
        })
        .await?;
    Ok(())
}
