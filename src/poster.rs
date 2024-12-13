use bsky_sdk::{
    api::{
        app::bsky::{
            embed::images::{ImageData, MainData},
            feed::{
                get_author_feed::ParametersData,
                post::{RecordData, RecordEmbedRefs},
            },
        },
        com::atproto::server::create_session::OutputData,
        types::{string::Datetime, Object, Union, Unknown},
    },
    BskyAgent,
};
use chrono::{DateTime, Duration, Timelike, Utc};
use chrono_tz::Europe::Helsinki;

use crate::{Aggregates, Title};

pub async fn check_post_exists(
    agent: &BskyAgent,
    session: &Object<OutputData>,
    title: &Title,
) -> Result<bool, Box<dyn std::error::Error>> {
    let own_posts_feed = agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(
            ParametersData {
                actor: session.data.handle.clone().into(),
                cursor: None,
                filter: None,
                include_pins: None,
                limit: Some(5.try_into().unwrap()),
            }
            .into(),
        )
        .await?;

    let posts_with_date =
        own_posts_feed
            .data
            .feed
            .iter()
            .find(|record| match &record.data.post.data.record {
                Unknown::Object(r) => {
                    match r.get_key_value("text") {
                        Some((_, x)) => {
                            if format!("{:?}", x).contains(&title.date) {
                                return true;
                            }
                        }
                        _ => {}
                    };
                    false
                }
                _ => false,
            });

    Ok(posts_with_date.is_some())
}

pub async fn post(
    agent: &BskyAgent,
    image_filename: &str,
    aggregates: &Aggregates,
    title: &Title,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let text_avg = format!("Keskihinta: {:.2} c/kWh", aggregates.avg).replace(".", ",");
    let text_min = format!(
        "Minimi: {:.2} c/kWh (klo {})",
        aggregates.min.1,
        timerange(aggregates.min.0)
    )
    .replace(".", ",");
    let text_max = format!(
        "Maksimi: {:.2} c/kWh (klo {})",
        aggregates.max.1,
        timerange(aggregates.max.0)
    )
    .replace(".", ",");
    let text_vat = "Hinnat sisältävät alv. 25,5%";

    let text = format!(
        "{}\n\n{}\n{}\n{}\n\n{}",
        text_header, text_avg, text_min, text_max, text_vat
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
