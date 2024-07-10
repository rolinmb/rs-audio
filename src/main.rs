extern crate hound;
use std::io;
use std::io::{/*Read, Write,*/ BufReader, BufWriter};
use std::path::Path;
use std::fs;
use std::fs::File;
use std::f32::consts::PI;

const AUDIO_OUT: &str = "audio_out";

fn clear_directory(dir_name: &str) -> io::Result<()> {
  let dir = Path::new(dir_name);
  if dir.exists() {
    println!("clean_directory(): Cleaning directory {}", dir_name);
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_dir() {
        fs::remove_dir_all(&path)?;
      } else {
        fs::remove_file(&path)?;
      }
    }
  } else {
    println!("clean_directory(): Creating directory {}", dir_name);
    fs::create_dir_all(dir)?;
  }
  println!("clean_directory(): Successfully created/cleaned directory {}", dir_name);
  Ok(())
}

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
  let spec = reader.spec();
  if spec.channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "half_time(): The input file is not stereo"));
  }
  let mut writer = hound::WavWriter::new(writer, spec).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let samples = reader.samples::<i16>();
  let mut sample_iter = samples.map(|s| s.map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
  while let (Some(l_sample), Some(r_sample)) = (sample_iter.next(), sample_iter.next()) {
    let l_sample = l_sample?;
    let r_sample = r_sample?;
    let mut f_lsample = l_sample as f32;
    let mut f_rsample = r_sample as f32;
    f_lsample = f_lsample.max(-32768.0).min(32767.0);
    f_rsample = f_rsample.max(-32768.0).min(32767.0);
    writer.write_sample(f_lsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(f_rsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(f_lsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(f_rsample as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  println!("half_time(): Successfully applied");
  Ok(())
}

fn apply_distortion(f_in: &mut File, f_out: &mut File, dist_val: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  if reader.spec().channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "apply_distortion(): The input file is not stereo"));
  }
  let samples = reader.samples::<i16>();
  let mut sample_iter = samples.map(|s| s.map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
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
  println!("apply_distortion(): Successfully applied");
  Ok(())
}

fn apply_reverb(f_in: &mut File, f_out: &mut File, time: usize, decay: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut writer = hound::WavWriter::new(writer, reader.spec()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  if reader.spec().channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "apply_reverb(): The input file is not stereo"));
  }
  let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
  let mut processed_samplesl: Vec<f32> = vec![0.0; samples.len() / 2 + time];
  let mut processed_samplesr: Vec<f32> = vec![0.0; samples.len() / 2 + time];
  for i in 0..(samples.len() / 2) {
    let l_sample = samples[2 * i] as f32;
    let r_sample = samples[2 * i + 1] as f32;
    processed_samplesl[i] += l_sample;
    processed_samplesr[i] += r_sample;
    if i + time < processed_samplesl.len() {
      processed_samplesl[i + time] += l_sample * decay;
      processed_samplesr[i + time] += r_sample * decay;
    }
  }
  for i in 0..(samples.len() / 2) {
    writer.write_sample(processed_samplesl[i].max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(processed_samplesr[i].max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  println!("apply_reverb(): Successfully applied");
  Ok(())
}

fn apply_chorus(f_in: &mut File, f_out: &mut File, delay: usize, depth: f32, mod_rate: f32) -> io::Result<()> {
  let reader = BufReader::new(f_in);
  let writer = BufWriter::new(f_out);
  let mut reader = hound::WavReader::new(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let spec = reader.spec();
  if spec.channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "apply_chorus(): The input file is not stereo"));
  }
  let mut writer = hound::WavWriter::new(writer, spec).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
  let sample_rate = spec.sample_rate as f32;
  let mut delay_bufferl: Vec<f32> = vec![0.0; delay];
  let mut delay_bufferr: Vec<f32> = vec![0.0; delay];
  let mut delay_idxl = 0;
  let mut delay_idxr = 0;
  let mut l_phase = 0.0;
  let mut r_phase = PI;
  for i in 0..(samples.len() / 2) {
    let l_sample = samples[2 * i] as f32;
    let r_sample = samples[2 * i + 1] as f32;
    let l_mod = (depth / 2.0) * (l_phase * 2.0 * PI / sample_rate).sin(); // Phase Modulation
    let r_mod = (depth / 2.0) * (r_phase * 2.0 * PI / sample_rate).sin();
    delay_bufferl[delay_idxl] = l_sample + l_mod;
    delay_bufferr[delay_idxr] = r_sample + r_mod;
    let delayed_samplel = delay_bufferl[(delay_idxl + delay - (delay as f32 + l_mod) as usize) % delay]; // Delay
    let delayed_sampler = delay_bufferr[(delay_idxr + delay - (delay as f32 + r_mod) as usize) % delay];
    writer.write_sample(delayed_samplel.max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(delayed_sampler.max(-32768.0).min(32767.0) as i16).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    delay_idxl = (delay_idxl + 1) % delay;
    delay_idxr = (delay_idxr + 1) % delay;
    l_phase += mod_rate;
    r_phase += mod_rate;
  }
  println!("apply_chorus(): Successfully applied");
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
  if spec.channels != 2 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "apply_bitcrush(): The input file is not stereo"));
  }
  let mut writer = hound::WavWriter::new(writer, spec).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let max_sample_val = 2.0f32.powi((spec.bits_per_sample as i32) - 1) - 1.0;
  let step_size = 2.0f32.powi((spec.bits_per_sample as i32) - (bits as i32));
  let samples = reader.samples::<i16>();
  let mut sample_iter = samples.map(|s| s.map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
  while let (Some(l_sample), Some(r_sample)) = (sample_iter.next(), sample_iter.next()) {
    let l_sample = l_sample?;
    let r_sample = r_sample?;
    let ql_sample = ((l_sample as f32 / max_sample_val * step_size).round() * max_sample_val / step_size) as i16;
    let qr_sample = ((r_sample as f32 / max_sample_val * step_size).round() * max_sample_val / step_size) as i16;
    writer.write_sample(ql_sample).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writer.write_sample(qr_sample).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  }
  println!("apply_bitcrush(): Successfully applied");
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
  clear_directory(AUDIO_OUT)?;
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
  /*let mut output_file1 = File::open("audio_out/next_fx_07092024_dht.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file2 = File::create("audio_out/next_fx_07092024_dhtr.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_reverb(&mut output_file1, &mut output_file2, 44100, 0.5)?;
  drop(output_file1);
  drop(output_file2);*/
  /*let mut output_file2 = File::open("audio_out/next_fx_07092024_dhtr.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file3 = File::create("audio_out/next_fx_07092024_dhtrd.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_delay(&mut output_file2, &mut output_file3, 44100, 0.25)?;
  drop(output_file2);
  drop(output_file3);*/
  let mut output_file1 = File::open("audio_out/next_fx_07092024_dht.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file2 = File::create("audio_out/next_fx_07092024_dhtb.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_bitcrush(&mut output_file1, &mut output_file2, 3)?;
  drop(output_file1);
  drop(output_file2);
  let mut output_file2 = File::open("audio_out/next_fx_07092024_dhtb.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  let mut output_file3 = File::create("audio_out/next_fx_07092024_dhtbc.wav").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
  apply_chorus(&mut output_file2, &mut output_file3, (0.1 * 44100), 0.75, 0.5)?;
  drop(output_file2);
  drop(output_file3);
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
