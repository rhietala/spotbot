use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Europe::Helsinki;
use dotenv::dotenv;
use std::env;

mod entsoe;
mod parser;
mod plotter;
mod poster;

static PLOT_FILENAME: &str = "prices.png";

#[derive(Clone, Debug)]
struct Aggregates {
    pub min: (DateTime<Utc>, f32),
    pub max: (DateTime<Utc>, f32),
    pub avg: f32,
}

#[derive(Clone, Debug)]
struct Title {
    pub weekday: String,
    pub date: String,
}

fn get_title(date: &NaiveDate) -> Title {
    let weekday = match date.weekday() {
        chrono::Weekday::Mon => "maanantai",
        chrono::Weekday::Tue => "tiistai",
        chrono::Weekday::Wed => "keskiviikko",
        chrono::Weekday::Thu => "torstai",
        chrono::Weekday::Fri => "perjantai",
        chrono::Weekday::Sat => "lauantai",
        chrono::Weekday::Sun => "sunnuntai",
    };
    let date = format!("{}.{}.{}", date.day(), date.month(), date.year(),);

    Title {
        weekday: weekday.to_string(),
        date,
    }
}

fn calculate_aggregates(prices: &[(DateTime<Utc>, f32)]) -> Aggregates {
    let min = *prices
        .iter()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = *prices
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let avg = prices.iter().map(|(_, a)| a).sum::<f32>() / prices.len() as f32;

    Aggregates { min, max, avg }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let day = (Utc::now() + Duration::days(1))
        .with_timezone(&Helsinki)
        .date_naive();

    println!("Starting spotbot for day {}", day);

    dotenv().ok();
    let entsoe_apikey = env::var("ENTSOE_APIKEY").expect("ENTSOE_APIKEY must be set");
    let bluesky_username = env::var("BLUESKY_USERNAME").expect("BLUESKY_USERNAME must be set");
    let bluesky_password = env::var("BLUESKY_PASSWORD").expect("BLUESKY_PASSWORD must be set");

    println!("Fetching prices from entsoe");
    let prices_xml = entsoe::get_spot_prices(&entsoe_apikey, day).await;

    println!("Parsing prices");
    let prices = parser::parse_xml(prices_xml);

    assert!(!prices.is_empty());

    let prices = prices
        .into_iter()
        .filter(|(ts, _)| ts.with_timezone(&Helsinki).date_naive() == day)
        .map(|(ts, price)| (ts, price * 100.0 / 1000.0)) // convert €/MWh to c / kWh
        .collect::<Vec<_>>();

    let aggregates = calculate_aggregates(&prices);
    let title = get_title(&day);

    println!("Plotting graph");
    plotter::plot(PLOT_FILENAME, &prices, &aggregates, &title).unwrap();

    println!("Posting to bluesky");
    poster::post(
        &bluesky_username,
        &bluesky_password,
        PLOT_FILENAME,
        &aggregates,
        &title,
    )
    .await?;

    Ok(())
}
