use bsky_sdk::BskyAgent;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Europe::{Helsinki, Riga, Stockholm, Tallinn, Vilnius};
use chrono_tz::{Tz, CET};
use dotenv::dotenv;
use poster::check_post_exists;
use std::env;
use std::str::FromStr;

mod currencies;
mod entsoe;
mod parser;
mod plotter;
mod poster;

#[derive(Clone, Debug)]
struct Aggregates {
    pub min: (DateTime<Utc>, f32),
    pub max: (DateTime<Utc>, f32),
    pub avg: f32,
}

#[derive(Clone, Debug)]
struct Localization {
    timezone: Tz,
    chrono_locale: chrono::Locale,
    day_name_locale: chrono::Locale,
    num_locale: num_format::Locale,
    plot_y_desc: &'static str,
    plot_x_desc: &'static str,
    plot_limit_multiplier: f32,
    post_title: &'static str,
    post_at: &'static str,
    post_avg: &'static str,
    post_min: &'static str,
    post_max: &'static str,
    post_vat: &'static str,
    currency_name: &'static str,
    currency_multiplier: f32,
}

fn get_localization(locale: chrono::Locale, currencies: currencies::Currencies) -> Localization {
    match locale {
        chrono::Locale::fi_FI => Localization {
            timezone: Helsinki,
            chrono_locale: locale,
            day_name_locale: locale,
            num_locale: num_format::Locale::fi,
            plot_y_desc: "hinta c/kWh",
            plot_x_desc: "tunti",
            plot_limit_multiplier: 1.0,
            post_title: "Pörssisähkön spot-hinnat",
            post_at: "klo",
            post_avg: "Keskiarvo",
            post_min: "Minimi",
            post_max: "Maksimi",
            post_vat: "Hinnat sisältävät alv.",
            currency_name: "c",
            currency_multiplier: currencies.eur_c,
        },
        chrono::Locale::et_EE => Localization {
            timezone: Tallinn,
            chrono_locale: chrono::Locale::et_EE,
            day_name_locale: chrono::Locale::en_GB,
            num_locale: num_format::Locale::et,
            plot_y_desc: "price c/kWh",
            plot_x_desc: "hour",
            plot_limit_multiplier: 1.0,
            post_title: "Electricity spot price",
            post_at: "at",
            post_avg: "Average",
            post_min: "Minimum",
            post_max: "Maximum",
            post_vat: "",
            currency_name: "c",
            currency_multiplier: currencies.eur_c,
        },
        chrono::Locale::lt_LT => Localization {
            timezone: Vilnius,
            chrono_locale: locale,
            day_name_locale: chrono::Locale::en_GB,
            num_locale: num_format::Locale::lt,
            plot_y_desc: "price c/kWh",
            plot_x_desc: "hour",
            plot_limit_multiplier: 1.0,
            post_title: "Electricity spot price",
            post_at: "at",
            post_avg: "Average",
            post_min: "Minimum",
            post_max: "Maximum",
            post_vat: "",
            currency_name: "c",
            currency_multiplier: currencies.eur_c,
        },
        chrono::Locale::lv_LV => Localization {
            timezone: Riga,
            chrono_locale: locale,
            day_name_locale: chrono::Locale::en_GB,
            num_locale: num_format::Locale::lv,
            plot_y_desc: "price c/kWh",
            plot_x_desc: "hour",
            plot_limit_multiplier: 1.0,
            post_title: "Electricity spot price",
            post_at: "at",
            post_avg: "Average",
            post_min: "Minimum",
            post_max: "Maximum",
            post_vat: "",
            currency_name: "c",
            currency_multiplier: currencies.eur_c,
        },
        chrono::Locale::sv_SE => Localization {
            timezone: Stockholm,
            chrono_locale: locale,
            day_name_locale: chrono::Locale::en_GB,
            num_locale: num_format::Locale::sv,
            plot_y_desc: "price öre/kWh",
            plot_x_desc: "hour",
            plot_limit_multiplier: 10.0,
            post_title: "Electricity spot price",
            post_at: "at",
            post_avg: "Average",
            post_min: "Minimum",
            post_max: "Maximum",
            post_vat: "",
            currency_name: "öre",
            currency_multiplier: currencies.sek_ore,
        },
        _ => Localization {
            timezone: CET,
            chrono_locale: locale,
            day_name_locale: chrono::Locale::en_GB,
            num_locale: num_format::Locale::en,
            plot_y_desc: "price c/kWh",
            plot_x_desc: "hour",
            plot_limit_multiplier: 1.0,
            post_title: "Electricity spot prices",
            post_at: "at",
            post_avg: "Average",
            post_min: "Minimum",
            post_max: "Maximum",
            post_vat: "",
            currency_name: "c",
            currency_multiplier: currencies.eur_c,
        },
    }
}

fn get_day_title(date: &NaiveDate, localization: &Localization) -> String {
    date.format_localized("%A", localization.day_name_locale)
        .to_string()
        + " "
        + &date
            .format_localized("%x", localization.chrono_locale)
            .to_string()
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
    dotenv().ok();
    let locale_str = env::var("SPOTBOT_LOCALE").expect("SPOTBOT_LOCALE must be set");
    let entsoe_apikey = env::var("ENTSOE_APIKEY").expect("ENTSOE_APIKEY must be set");
    let entsoe_eic = env::var("ENTSOE_EIC").expect("ENTSOE_EIC must be set");
    let bluesky_username = env::var("BLUESKY_USERNAME").expect("BLUESKY_USERNAME must be set");
    let bluesky_password = env::var("BLUESKY_PASSWORD").expect("BLUESKY_PASSWORD must be set");
    let vat: f32 = env::var("VAT")
        .expect("VAT must be set")
        .parse::<f32>()
        .expect("VAT must be a number");

    let agent = BskyAgent::builder().build().await?;
    let session = agent.login(&bluesky_username, &bluesky_password).await?;
    let currencies = currencies::get_currencies().unwrap();
    let locale = chrono::Locale::from_str(&locale_str).expect("Invalid locale");

    let localization = get_localization(locale, currencies);
    let day = (Utc::now() + Duration::days(1))
        .with_timezone(&localization.timezone)
        .date_naive();
    let day_title = get_day_title(&day, &localization);

    println!("Starting {} for day {}", bluesky_username, day);

    if check_post_exists(&agent, &session, &day_title).await? {
        println!("Post already exists, skipping");
        return Ok(());
    }

    println!("Fetching prices from entsoe");
    let prices_xml =
        entsoe::get_spot_prices(&entsoe_apikey, &entsoe_eic, day, &localization.timezone).await;

    println!("Parsing prices");
    let prices = parser::parse_xml(prices_xml);

    let prices = prices
        .into_iter()
        .filter(|(ts, _)| ts.with_timezone(&localization.timezone).date_naive() == day)
        // convert €/MWh to <local currency>/kWh, apply VAT
        .map(|(ts, price)| {
            (
                ts,
                price * localization.currency_multiplier / 1000.0 * (vat / 100.0 + 1.0),
            )
        })
        .collect::<Vec<_>>();

    assert!(
        23 <= prices.len() && prices.len() <= 25,
        "Expected 23..25 price points, got {}",
        prices.len()
    );

    let aggregates = calculate_aggregates(&prices);

    let plot_filename = format!("{}-{}.png", &bluesky_username, &day.format("%Y-%m-%d"));

    println!("Plotting graph");
    plotter::plot(
        &plot_filename,
        &prices,
        &aggregates,
        &day_title,
        &localization,
    )
    .unwrap();

    println!("Posting to bluesky");
    poster::post(
        &agent,
        &plot_filename,
        &aggregates,
        &localization,
        &day_title,
        vat,
    )
    .await?;

    Ok(())
}
