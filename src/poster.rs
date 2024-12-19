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

use crate::{Aggregates, Localization};

pub async fn check_post_exists(
    agent: &BskyAgent,
    session: &Object<OutputData>,
    title: &String,
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
                            if format!("{:?}", x).contains(title) {
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
    localization: &Localization,
    day_title: &String,
    vat: f32,
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

    let text_header = format!("{} {}", localization.post_title, day_title);
    let text_avg = format!(
        "{}: {:.2} {}/kWh",
        localization.post_avg, aggregates.avg, localization.currency_name
    )
    .replace(".", localization.num_locale.decimal());
    let text_min = format!(
        "{}: {:.2} {}/kWh ({} {})",
        localization.post_min,
        aggregates.min.1,
        localization.currency_name,
        localization.post_at,
        timerange(aggregates.min.0)
    )
    .replace(".", localization.num_locale.decimal());
    let text_max = format!(
        "{}: {:.2} {}/kWh ({} {})",
        localization.post_max,
        aggregates.max.1,
        localization.currency_name,
        localization.post_at,
        timerange(aggregates.max.0)
    )
    .replace(".", localization.num_locale.decimal());

    let text_vat = if vat > 0.0 {
        format!(
            "\n\n{} {} %",
            localization.post_vat,
            format!("{:.1}", vat).replace(".", localization.num_locale.decimal())
        )
    } else {
        "".to_string()
    };

    let text = format!(
        "{}\n\n{}\n{}\n{}{}",
        text_header, text_avg, text_min, text_max, text_vat
    );

    let embed = Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(
        Box::new(MainData { images }.into()),
    )));

    println!("Posting: {}", text);

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
