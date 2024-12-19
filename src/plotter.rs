use std::ops::Range;

use chrono::{DateTime, Timelike, Utc};
use plotters::{define_color, doc, prelude::*};

use crate::{Aggregates, Localization};

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

pub fn calculate_limits(
    aggregates: &Aggregates,
    localization: &Localization,
) -> (Range<f32>, f32, f32) {
    let max_y = aggregates.max.1;
    let min_y = aggregates.min.1;

    let minimum_max_y = 20.0 * localization.plot_limit_multiplier;
    let max_y_step = 10.0 * localization.plot_limit_multiplier;
    let min_y_step = 5.0 * localization.plot_limit_multiplier;

    // round max_y to the nearest 10 above, or 20 at least
    let chart_max = if max_y > minimum_max_y {
        (max_y / max_y_step).ceil() * max_y_step
    } else {
        minimum_max_y
    };

    // round min_y to the nearest 5 below, or 0 if min_y is positive
    let chart_min = if min_y < 0.0 {
        (min_y / min_y_step).floor() * min_y_step
    } else {
        0.0
    };

    // min_y_step is used also as the cheap color limit
    (chart_min..chart_max, min_y_step, minimum_max_y)
}

pub fn plot(
    filename: &String,
    data: &[(DateTime<Utc>, f32)],
    aggregates: &Aggregates,
    title: &String,
    localization: &Localization,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let (chart_range, low, high) = calculate_limits(aggregates, localization);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(30)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d((0..23).into_segmented(), chart_range)?;

    let label_style = ("sans-serif", 25).into_font();

    chart
        .configure_mesh()
        .label_style(label_style.clone())
        .axis_desc_style(label_style)
        .y_desc(localization.plot_y_desc)
        .x_desc(localization.plot_x_desc)
        .x_labels(24)
        .y_labels(10)
        .y_max_light_lines(0)
        .y_label_formatter(&|y| format!("{:.0}", y))
        .draw()?;

    chart.draw_series(data.iter().map(|(ts, value)| {
        let x = ts.with_timezone(&localization.timezone).hour() as i32;

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

    root.present()?;

    Ok(())
}
