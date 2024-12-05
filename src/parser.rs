use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use xml::reader::{EventReader, XmlEvent};

fn on_start_element(
    name: &str,
    in_time_interval: &mut bool,
    in_point: &mut bool,
    current_element: &mut String,
) {
    *current_element = name.to_string();
    if name == "timeInterval" {
        *in_time_interval = true;
    } else if name == "Point" {
        *in_point = true;
    }
}

fn on_end_element(
    name: &str,
    in_time_interval: &mut bool,
    in_point: &mut bool,
    current_element: &mut String,
) {
    if name == "timeInterval" {
        *in_time_interval = false;
    } else if name == "Point" {
        *in_point = false;
    }
    current_element.clear();
}

fn on_characters(
    data: &str,
    in_time_interval: bool,
    in_point: bool,
    current_element: &str,
    current_timestamp: &mut Option<DateTime<Utc>>,
    step: &mut Option<Duration>,
    ret: &mut Vec<(DateTime<Utc>, f32)>,
) {
    if in_time_interval && (current_element == "start") {
        match NaiveDateTime::parse_from_str(data, "%Y-%m-%dT%H:%M%Z") {
            Ok(naive_time) => {
                *current_timestamp = Some(Utc.from_utc_datetime(&naive_time));
            }
            Err(e) => {
                println!("Error parsing timestamp: {}", e);
            }
        }
    } else if current_element == "resolution" {
        if data.trim() == "PT60M" {
            *step = Some(Duration::minutes(60));
        }
    } else if in_point && current_element == "price.amount" {
        let price: f32 = data.trim().parse().expect("Invalid price format");
        if let Some(timestamp) = *current_timestamp {
            ret.push((timestamp, price));
            if let Some(res) = *step {
                *current_timestamp = Some(timestamp + res);
            }
        }
    }
}

pub fn parse_xml(prices_xml: String) -> Vec<(DateTime<Utc>, f32)> {
    let parser = EventReader::new(prices_xml.as_bytes());

    let mut in_time_interval = false;
    let mut in_point = false;
    let mut current_element = String::new();
    let mut current_timestamp: Option<DateTime<Utc>> = None;
    let mut step: Option<Duration> = None;

    let mut ret: Vec<(DateTime<Utc>, f32)> = Vec::new();

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                on_start_element(
                    &name.local_name,
                    &mut in_time_interval,
                    &mut in_point,
                    &mut current_element,
                );
            }
            Ok(XmlEvent::EndElement { name }) => {
                on_end_element(
                    &name.local_name,
                    &mut in_time_interval,
                    &mut in_point,
                    &mut current_element,
                );
            }
            Ok(XmlEvent::Characters(data)) => {
                on_characters(
                    &data,
                    in_time_interval,
                    in_point,
                    &current_element,
                    &mut current_timestamp,
                    &mut step,
                    &mut ret,
                );
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    ret
}
