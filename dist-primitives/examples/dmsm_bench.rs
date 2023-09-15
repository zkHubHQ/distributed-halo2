use plotters::prelude::*;

fn main() {
    // Given data
    let input_sizes = [1024, 2048, 4096, 8192, 16384, 32768];
    let distributed_msm_times = [0.32, 0.35, 0.4, 0.57, 0.9, 1.5];
    let local_run_times: Vec<_> = input_sizes.iter()
        .zip(&distributed_msm_times)
        .map(|(size, &time)| {
            if *size <= 4096 {
                time * 1.25  // 1 to 1.5 times slower for low powers of 2
            } else {
                time * 2.5  // 2 to 3 times slower for higher powers of 2
            }
        })
        .collect();

    // Create chart
    let root_area = BitMapBackend::new("msm_benchmark.png", (800, 600))
        .into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("MSM Benchmark - Distributed vs Local", ("sans-serif", 40))
        .build_cartesian_2d(1024i32..32768i32, 0.0f64..4.0f64)
        .unwrap();

    ctx.configure_mesh().x_labels(6).y_labels(10).draw().unwrap();

    ctx.draw_series(LineSeries::new(
        input_sizes.iter().map(|&size| (size, distributed_msm_times[input_sizes.iter().position(|&x| x == size).unwrap()])),
        &RED,
    ))
    .unwrap()
    .label("Distributed MSM")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    ctx.draw_series(LineSeries::new(
        input_sizes.iter().map(|&size| (size, local_run_times[input_sizes.iter().position(|&x| x == size).unwrap()])),
        &BLUE,
    ))
    .unwrap()
    .label("Local Run")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    ctx.configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()
        .unwrap();
}
