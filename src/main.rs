extern crate hound;
//use std::fs;
use std::fs::File;
use std::io::{self, /*Read, Write,*/ BufReader, BufWriter};

/*enum Effect {
  Halftime,
  Distortion(f32),
  Reverb(usize, f32),
  //Delay(usize, f32),
  Bitcrush(u32),
}

impl Effect {
  fn apply(&self, f_in: &mut File, f_out: &mut File) -> io::Result<()> {
    match self {
      Effect::Halftime => half_time(f_in, f_out),
      Effect::Distortion(dist_val) => apply_distortion(f_in, f_out, *dist_val),
      Effect::Reverb(time, decay) => apply_reverb(f_in, f_out, *time, *decay),
      //Effect::Delay(nsamples, decay) => apply_delay(f_in, f_out, *nsamples, *decay),
      Effect::Bitcrush(bits) => apply_bitcrush(f_in, f_out, *bits),
    }
  }
}*/

/*fn copy_file(src: &mut File, dst: &mut File) -> io::Result<()> {
  let mut buffer = Vec::new();
  src.read_to_end(&mut buffer)?;
  dst.write_all(&buffer)?;
  Ok(())
}*/

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

fn apply_reverb(f_in: &mut File, f_out: &mut File, time: usize, decay: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
  let mut processed_samples: Vec<f32> = vec![0.0; samples.len() + time];
  for i in 0..samples.len() {
    processed_samples[i] += samples[i] as f32;
    if i + time < processed_samples.len() {
      processed_samples[i + time] += (samples[i] as f32) * decay;
    }
  }
  for sample in processed_samples {
    writer.write_sample(sample.max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}

//TODO: troubleshoot why this never stops / corrupts data
/*fn apply_delay(f_in: &mut File, f_out: &mut File, delay_samples: usize, decay: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut delay_buffer: Vec<f32> = vec![0.0; delay_samples];
  for sample in reader.samples::<i16>() {
    let sample = sample.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut delayed_sample = sample as f32;
    let delayed_val = delay_buffer.remove(0);
    delay_buffer.push(delayed_sample + decay * delayed_val);
    writer.write_sample(delayed_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}*/

fn apply_bitcrush(f_in: &mut File, f_out: &mut File, bits: u32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let spec = reader.spec();
  let mut writer = hound::WavWriter::new(writer, spec).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let max_sample_val = 2.0f32.powi((spec.bits_per_sample as i32) - 1) - 1.0;
  let step_size = 2.0f32.powi((spec.bits_per_sample as i32) - (bits as i32));
  for sample in reader.samples::<i16>() {
    let sample = sample.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let f_sample = sample as f32;
    let quantized_sample = (f_sample / max_sample_val * step_size).round() * max_sample_val / step_size;
    writer.write_sample(quantized_sample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  Ok(())
}

//TODO: fix this so I can use the enum defined at top of file to pass a list of FX to the audio file
/*fn apply_effects(f_in: &mut File, f_out: &mut File, effects: &[Effect]) -> io::Result<()> {
  let mut temp_files: Vec<String> = Vec::new();
  let mut temp_in = f_in;
  let mut new_temp_in = None;
  for (i, fx) in effects.iter().enumerate() {
      let temp_out_path = format!("audio_out/temp_{}.wav", i);
      let mut temp_out = File::create(&temp_out_path)?;
      fx.apply(temp_in, &mut temp_out)?;
      temp_files.push(temp_out_path.clone());
      drop(temp_out);
      new_temp_in = Some(File::open(&temp_out_path)?);
      std::mem::swap(&mut temp_in, new_temp_in.as_mut().unwrap());
      fs::remove_file(temp_out_path)?;
  }
  copy_file(temp_in, f_out)?;
  for temp_file in temp_files {
    fs::remove_file(temp_file)?;
  }
  Ok(())
}*/

fn main() -> io::Result<()> {
  let mut input_file = File::open("audio_in/next.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file0 = File::create("audio_out/next_fx_07092024_d.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_distortion(&mut input_file, &mut output_file0, 444.0)?;
  drop(input_file);
  drop(output_file0);
  let mut output_file0 = File::open("audio_out/next_fx_07092024_d.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file1 = File::create("audio_out/next_fx_07092024_dht.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  half_time(&mut output_file0, &mut output_file1)?;
  drop(output_file0);
  drop(output_file1);
  let mut output_file1 = File::open("audio_out/next_fx_07092024_dht.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file2 = File::create("audio_out/next_fx_07092024_dhtr.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_reverb(&mut output_file1, &mut output_file2, 44100, 0.5)?;
  drop(output_file1);
  drop(output_file2);
  let mut output_file2 = File::open("audio_out/next_fx_07092024_dhtr.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file3 = File::create("audio_out/next_fx_07092024_dhtrd.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_delay(&mut output_file2, &mut output_file3, 44100, 0.25)?;
  drop(output_file2);
  drop(output_file3);
  /*let mut output_file3 = File::open("audio_out/next_fx_07092024_dhtrd.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file4 = File::create("audio_out/next_fx_07092024_dhtrdb.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_bitcrush(&mut output_file3, &mut output_file4, 7)?;
  drop(output_file3);
  drop(output_file4);*/
  /*let mut input_file = File::open("audio_in/next.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file5 = File::create("audio_out/next_fx_07092024_fxlist.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let effects_list = vec![
    Effect::Distortion(444.0),
    Effect::Halftime,
    Effect::Reverb(44100, 0.5),
    Effect::Delay(44100, 0.5),
    Effect::Bitcrush(7),
  ];
  apply_effects(&mut input_file, &mut output_file4, &effects_list)?;
  drop(input_file);
  drop(output_file5);*/
  Ok(())
}
