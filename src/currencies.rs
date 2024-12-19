use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
struct ExchangeRateCurrency {
    code: String,
    value: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExchangeRateData {
    #[serde(rename = "DKK")]
    dkk: ExchangeRateCurrency,
    #[serde(rename = "NOK")]
    nok: ExchangeRateCurrency,
    #[serde(rename = "SEK")]
    sek: ExchangeRateCurrency,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExchangeRate {
    data: ExchangeRateData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currencies {
    // base currency is EUR
    pub eur_c: f32,
    pub dkk: f32,
    pub nok: f32,
    pub sek_ore: f32,
}

pub fn get_currencies() -> Result<Currencies> {
    // Open the JSON file
    let file = File::open("exchange-rates.json").expect("File not found");
    let reader = BufReader::new(file);

    let exchange_rate: ExchangeRate = serde_json::from_reader(reader)?;

    let rates = Currencies {
        eur_c: 100.0,
        dkk: exchange_rate.data.dkk.value,
        nok: exchange_rate.data.nok.value,
        sek_ore: exchange_rate.data.sek.value * 100.0,
    };

    Ok(rates)
}
