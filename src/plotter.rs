use std::ops::Range;

use chrono::{DateTime, Timelike, Utc};
use chrono_tz::Europe::Helsinki;
use plotters::{define_color, doc, prelude::*};

use crate::{Aggregates, Title};

define_color!(NORMAL, 211, 210, 71, "Normal price color");
define_color!(HIGH, 211, 186, 71, "High");
define_color!(LOW, 155, 197, 66, "Low");

fn color_by_value(value: f32, high: f32, low: f32) -> ShapeStyle {
    if value > high {
        HIGH.filled()
    } else if value < low {
        LOW.filled()
    } else {
        NORMAL.filled()
    }
}

pub fn calculate_limits(aggregates: &Aggregates) -> (Range<f32>, f32, f32) {
    let max_y = aggregates.max.1;
    let min_y = aggregates.min.1;
    // round max_y to the nearest 10 above, or 20 at least
    let chart_max = if max_y > 20.0 {
        (max_y / 10.0).ceil() * 10.0
    } else {
        20.0
    };

    // round min_y to the nearest 10 below, or 0 if min_y is positive
    let chart_min = if min_y < 0.0 {
        (min_y / 10.0).floor() * 10.0
    } else {
        0.0
    };

    (chart_min..chart_max, max_y * 0.2, max_y * 0.8)
}

pub fn plot(
    filename: &str,
    data: &[(DateTime<Utc>, f32)],
    aggregates: &Aggregates,
    title: &Title,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let (chart_range, low, high) = calculate_limits(aggregates);

    // Create a chart builder
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} {}", title.weekday, title.date),
            ("sans-serif", 50).into_font(),
        )
        .margin(30)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d((0..23).into_segmented(), chart_range)?;

    let label_style = ("sans-serif", 25).into_font();

    // Configure the mesh
    chart
        .configure_mesh()
        .label_style(label_style.clone())
        .axis_desc_style(label_style)
        .y_desc("hinta c/kWh")
        .x_desc("tunti")
        .x_labels(24)
        .y_labels(10)
        .y_max_light_lines(0)
        .y_label_formatter(&|y| format!("{:.0}", y))
        .draw()?;

    // Plot the data
    chart.draw_series(data.iter().map(|(ts, value)| {
        let x = ts.with_timezone(&Helsinki).hour() as i32;

        let mut bar = Rectangle::new(
            [
                (SegmentValue::Exact(x), 0.0),
                (SegmentValue::Exact(x + 1), *value),
            ],
            color_by_value(*value, high, low),
        );
        bar.set_margin(0, 0, 2, 2);
        bar
    }))?;

    // Save the result
    root.present()?;

    Ok(())
}
