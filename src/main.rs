extern crate hound;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

fn half_time(f_in: &mut File, f_out: &mut File) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  for sample in reader.samples::<i16>() {
    let sample = sample.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut f_sample = sample as f32;
    f_sample = f_sample.max(-32768.0).min(32767.0);
    writer.write_sample(f_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(f_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}

fn apply_distortion(f_in: &mut File, f_out: &mut File, dist_val: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  if reader.spec().channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "The input file is not stereo"));
  }
  let samples = reader.samples::<i16>();
  let mut sample_iter = samples.map(|sample| sample.map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
  while let (Some(l_sample), Some(r_sample)) = (sample_iter.next(), sample_iter.next()) {
    let l_sample = l_sample?;
    let r_sample = r_sample?;
    let mut fx_lsample = (l_sample as f32) * dist_val;
    let mut fx_rsample = (r_sample as f32) * dist_val;
    fx_lsample = fx_lsample.max(-32768.0).min(32767.0);
    fx_rsample = fx_rsample.max(-32768.0).min(32767.0);
    writer.write_sample(fx_lsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(fx_rsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}

fn apply_reverb(f_in: &mut File, f_out: &mut File, delay: usize, decay: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let spec = reader.spec();
  let mut writer = hound::WavWriter::new(writer, spec).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
  let mut processed_samples: Vec<f32> = vec![0.0; samples.len() + delay];
  for i in 0..samples.len() {
    processed_samples[i] += samples[i] as f32;
    if i + delay < processed_samples.len() {
      processed_samples[i + delay] += (samples[i] as f32) * decay;
    }
  }
  for sample in processed_samples {
    writer.write_sample(sample.max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}

fn main() -> io::Result<()> {
  let mut input_file = File::open("audio_in/next.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file0 = File::create("audio_out/next_fx_07092024_dist.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_distortion(&mut input_file, &mut output_file0, 444.0)?;
  drop(input_file);
  drop(output_file0);
  let mut output_file0 = File::open("audio_out/next_fx_07092024_dist.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file1 = File::create("audio_out/next_fx_07092024_ht.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  half_time(&mut output_file0, &mut output_file1)?;
  drop(output_file0);
  drop(output_file1);
  let mut output_file1 = File::open("audio_out/next_fx_07092024_dist.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file2 = File::create("audio_out/next_fx_07092024_reverb.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_reverb(&mut output_file1, &mut output_file2, 44100, 0.5)?;
  drop(output_file1);
  drop(output_file2);
  Ok(())
}
