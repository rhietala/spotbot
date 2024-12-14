use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

static ENTSOE_URL: &str = "https://web-api.tp.entsoe.eu/api";

pub async fn get_spot_prices(apikey: &str, eic: &str, day: NaiveDate, timezone: &Tz) -> String {
    let start_of_day = day.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let start = timezone.from_local_datetime(&start_of_day).unwrap();

    let end = start.with_timezone(timezone) + Duration::days(1);
    let period_start = start.with_timezone(&Utc).format("%Y%m%d%H%M").to_string();
    let period_end = end.with_timezone(&Utc).format("%Y%m%d%H%M").to_string();
    let params: Vec<(&str, String)> = vec![
        ("documentType", "A44".to_string()),
        ("contract_MarketAgreement.type", "A01".to_string()),
        ("periodStart", period_start),
        ("periodEnd", period_end),
        ("out_Domain", eic.to_string()),
        ("in_Domain", eic.to_string()),
        ("securityToken", apikey.to_string()),
    ];
    let client = reqwest::Client::new();
    let res = client.get(ENTSOE_URL).query(&params).send().await.unwrap();
    res.text().await.unwrap()
}
