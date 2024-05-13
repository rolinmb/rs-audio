extern crate hound;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

fn main() -> io::Result<()> {
  let input_file = File::open("audio_in/next.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let reader = BufReader::new(input_file);
  let output_file = File::create("audio_out/next_fx_05132024.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let writer = BufWriter::new(output_file);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  for sample in reader.samples::<i16>() {
    let sample = sample.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let left_sample = sample as f32;
    let right_sample = sample as f32;
    let mut processed_left_sample = left_sample * 20.0;
    let mut processed_right_sample = right_sample * 20.0;
    processed_left_sample = processed_left_sample.max(-32768.0).min(32767.0);
    processed_right_sample = processed_right_sample.max(-32768.0).min(32767.0);
    writer.write_sample(processed_left_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(processed_right_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}
