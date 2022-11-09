use std::env;
use std::error::Error;
use std::fs::File;
use std::iter::once;
use std::path::Path;

use plotters::prelude::*;
use wav::BitDepth;

fn chart_wav(
    file: &str,
    sample_rate: u32,
    data: &Vec<i16>,
    start: f64,
    end: f64,
) -> Result<(), Box<dyn Error>> {
    //convert i16 to f64
    //let data: Vec<f64> = data.iter().map(|x| *x as f64).collect();

    let root = BitMapBackend::new(file, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Juice", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(start..end, -40000f64..40000f64)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        data.iter()
            .enumerate()
            .map(|(i, d)| (i as f64 / sample_rate as f64, *d as f64))
            .filter(|(x, _)| *x > start)
            .take_while(|(x, _)| *x < end)
            .map(|(x, y)| Circle::new((x, y), 1, RED.filled())),
    )?;

    root.present()?;

    Ok(())
}

fn construct_channels(data: &BitDepth) -> (Vec<i16>, Vec<i16>) {
    let data: Vec<i16> = data
        .as_sixteen()
        .unwrap()
        .iter()
        .map(|x| *x as i16)
        .collect();

    let left_channel: Vec<i16> = data.iter().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, x)| *x).collect();
    let right_channel: Vec<i16> = data.iter().enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, x)| *x).collect();

    (left_channel, right_channel)
}

fn finite_difference(header: &wav::Header, data: &BitDepth) -> Result<BitDepth, Box<dyn Error>> {
    let (left_channel, right_channel) = construct_channels(data);

    let left_diff: Vec<i16> = left_channel.windows(2).map(|w| (w[1] - w[0])).collect();
    let right_diff: Vec<i16> = right_channel.windows(2).map(|w| (w[1] - w[0])).collect();

    //reconstruct data by joining left and right channels
    let diff_data: Vec<i16> = left_diff.into_iter().zip(right_diff).flat_map(|(l, r)| once(l).chain(once(r))).collect();

    Ok(wav::BitDepth::Sixteen(diff_data))
}

fn spectral_difference(header: &wav::Header, data: &BitDepth) -> Result<BitDepth, Box<dyn Error>> {

}

fn main() -> Result<(), Box<dyn Error>> {
    //do this as paths
    let file_path_arg = match env::args().skip(1).next() {
        Some(file_name) => file_name,
        None => panic!("Expected an input file path"),
    };

    let in_file_path = Path::new(&file_path_arg);
    let file_name = match in_file_path.file_name() {
        Some(file_name) => file_name,
        None => panic!("Expected argument to a file not a directory"),
    };

    let mut inp_file = File::open(in_file_path)?;
    let (header, data) = wav::read(&mut inp_file)?;

    let bdepth = finite_difference(&header, &data)?;
    let mut out_file = File::create(Path::new(&format!("derivative_{}", file_name.to_string_lossy())))?;
    wav::write(header, &bdepth, &mut out_file)?;
    Ok(())
}
