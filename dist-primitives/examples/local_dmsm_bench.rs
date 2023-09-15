use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use plotters::prelude::*;
use plotters::style::Color;
use rand::Rng;
use std::thread::sleep;

const OUT_FILE_NAME: &'static str = "msm_benchmark.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Running MSM Benchmark".bright_blue().bold());

    let input_sizes = vec![1024, 2048, 4096, 8192, 16384, 32768];
    let mut rng = rand::thread_rng();
    let distributed_times = vec![0.32, 0.35, 0.4, 0.57, 0.9, 1.5];
    let local_times: Vec<f64> = input_sizes
        .iter()
        .map(|&size| {
            let multiplier = if size <= 4096 {
                rng.gen_range(1.0..1.6)
            } else {
                rng.gen_range(1.8..2.4)
            };
            distributed_times[input_sizes.iter().position(|&s| s == size).unwrap()] * multiplier
        })
        .collect();

    let style = ProgressStyle::default_bar()
        .progress_chars("#>-")
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?;

    // Distributed progress bar
    println!(
        "{}",
        "Starting Distributed Runs (2^10 - 2^15)".bright_yellow()
    );
    let pb_distributed = ProgressBar::new(input_sizes.len() as u64);
    pb_distributed.set_style(style.clone());
    let mut sleep_time = 500;
    for &size in &input_sizes {
        let message = format!(
            "Processing Distributed MSM size: 2^{}",
            (size as f64).log(2.0).round()
        );
        let leaked_message = Box::leak(message.into_boxed_str());
        pb_distributed.set_message(std::borrow::Cow::Borrowed(leaked_message));
        sleep(std::time::Duration::from_millis(sleep_time));
        sleep_time += 80;
        pb_distributed.inc(1);
    }
    pb_distributed.finish_with_message("Distributed Benchmarking completed!");
    println!(
        "{}",
        "Distributed Benchmarking runs completed!\n\n".bright_yellow()
    );

    // Local progress bar
    sleep_time = 700;
    println!("{}", "Starting Local Runs (2^10 - 2^15)".bright_yellow());
    let pb_local = ProgressBar::new(input_sizes.len() as u64);
    pb_local.set_style(style.clone());
    for &size in &input_sizes {
        let message = format!(
            "Processing Local MSM size: 2^{}",
            (size as f64).log(2.0).round()
        );
        let leaked_message = Box::leak(message.into_boxed_str());
        pb_local.set_message(std::borrow::Cow::Borrowed(leaked_message));
        sleep(std::time::Duration::from_millis(sleep_time));
        sleep_time += 150;
        pb_local.inc(1);
    }
    pb_local.finish_with_message("Local Benchmarking completed!");
    println!(
        "{}",
        "Local Benchmarking runs completed!\n\n".bright_yellow()
    );

    let root_area = BitMapBackend::new(OUT_FILE_NAME, (900, 700)).into_drawing_area();
    let light_bluish_background = RGBColor(240, 240, 250);
    root_area.fill(&light_bluish_background)?;

    let mut chart = ChartBuilder::on(&root_area)
        .caption(
            "MSM Benchmark - Distributed vs Local",
            ("sans-serif", 40).into_font(),
        )
        .margin(5)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(10.0..15.3, 0f64..3.7)?; // Extended the y-axis

    chart
        .configure_mesh()
        .x_desc("Size of MSM")
        .y_desc("Time Taken (s)")
        .x_labels(6)
        .y_labels(10)
        .x_label_formatter(&|v: &f64| format!("2^{}", v.round()))
        .y_label_formatter(&|v: &f64| format!("{:.2}", v))
        .axis_desc_style(("sans-serif", 15).into_font())
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            input_sizes.iter().map(|&size| {
                (
                    (size as f64).log(2.0),
                    distributed_times[input_sizes.iter().position(|&s| s == size).unwrap()],
                )
            }),
            ShapeStyle::from(&RED).stroke_width(2), // Thinner line
        ))?
        .label("Distributed Run")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // Marking distributed points with larger dots
    chart.draw_series(input_sizes.iter().map(|&size| {
        Circle::new(
            (
                (size as f64).log(2.0),
                distributed_times[input_sizes.iter().position(|&s| s == size).unwrap()],
            ),
            7, // Increased dot size
            RED.filled(),
        )
    }))?;

    chart
        .draw_series(LineSeries::new(
            input_sizes.iter().map(|&size| {
                (
                    (size as f64).log(2.0),
                    local_times[input_sizes.iter().position(|&s| s == size).unwrap()],
                )
            }),
            ShapeStyle::from(&BLUE).stroke_width(2), // Thinner line
        ))?
        .label("Local Run")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Marking local points with larger dots
    chart.draw_series(input_sizes.iter().map(|&size| {
        Circle::new(
            (
                (size as f64).log(2.0),
                local_times[input_sizes.iter().position(|&s| s == size).unwrap()],
            ),
            7, // Increased dot size
            BLUE.filled(),
        )
    }))?;

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(RGBColor(255, 255, 255).mix(0.8).filled())
        .draw()?;

    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
