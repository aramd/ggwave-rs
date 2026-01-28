use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ggwave_rs::{default_parameters, GgWave, ProtocolId, SampleFormat};
use hound::{SampleFormat as HoundSampleFormat, WavReader, WavSpec, WavWriter};

#[derive(Parser)]
#[command(name = "ggwave", about = "Encode/decode data via audio waveforms")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encode a message into a WAV file
    Encode {
        /// Message to encode
        message: String,
        /// Output WAV file path
        output: PathBuf,
        /// Volume (0-100)
        #[arg(short, long, default_value = "25")]
        volume: i32,
        /// Protocol to use
        #[arg(short, long, default_value = "audible-fast")]
        protocol: Protocol,
    },
    /// Decode a message from a WAV file
    Decode {
        /// Input WAV file path
        input: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Protocol {
    AudibleNormal,
    AudibleFast,
    AudibleFastest,
    UltrasoundNormal,
    UltrasoundFast,
    UltrasoundFastest,
    DtNormal,
    DtFast,
    DtFastest,
    MtNormal,
    MtFast,
    MtFastest,
}

impl From<Protocol> for ProtocolId {
    fn from(p: Protocol) -> Self {
        match p {
            Protocol::AudibleNormal => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_NORMAL,
            Protocol::AudibleFast => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
            Protocol::AudibleFastest => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FASTEST,
            Protocol::UltrasoundNormal => ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_NORMAL,
            Protocol::UltrasoundFast => ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
            Protocol::UltrasoundFastest => ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FASTEST,
            Protocol::DtNormal => ProtocolId::GGWAVE_PROTOCOL_DT_NORMAL,
            Protocol::DtFast => ProtocolId::GGWAVE_PROTOCOL_DT_FAST,
            Protocol::DtFastest => ProtocolId::GGWAVE_PROTOCOL_DT_FASTEST,
            Protocol::MtNormal => ProtocolId::GGWAVE_PROTOCOL_MT_NORMAL,
            Protocol::MtFast => ProtocolId::GGWAVE_PROTOCOL_MT_FAST,
            Protocol::MtFastest => ProtocolId::GGWAVE_PROTOCOL_MT_FASTEST,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Encode {
            message,
            output,
            volume,
            protocol,
        } => {
            if let Err(e) = encode(&message, &output, volume, protocol) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Command::Decode { input } => {
            if let Err(e) = decode(&input) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn encode(
    message: &str,
    output: &PathBuf,
    volume: i32,
    protocol: Protocol,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut params = default_parameters();
    params.sampleFormatInp = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
    params.sampleFormatOut = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;

    let ggwave = GgWave::new(params)?;
    let waveform = ggwave.encode(message.as_bytes(), protocol.into(), volume)?;

    let sample_rate = params.sampleRateOut as u32;
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };

    let mut writer = WavWriter::create(output, spec)?;

    // Convert raw bytes to f32 samples and write
    for chunk in waveform.chunks_exact(4) {
        let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    println!(
        "Encoded {} bytes into {} ({} samples, {} Hz)",
        message.len(),
        output.display(),
        waveform.len() / 4,
        sample_rate
    );

    Ok(())
}

fn decode(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        return Err(format!("expected mono audio, got {} channels", spec.channels).into());
    }

    // Read samples and convert to raw F32 bytes
    let waveform: Vec<u8> = match (spec.sample_format, spec.bits_per_sample) {
        (HoundSampleFormat::Float, 32) => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|s| s.to_le_bytes())
            .collect(),
        (HoundSampleFormat::Int, 16) => reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|s| s as f32 / i16::MAX as f32)
            .flat_map(|s| s.to_le_bytes())
            .collect(),
        (HoundSampleFormat::Int, 32) => reader
            .samples::<i32>()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|s| s as f32 / i32::MAX as f32)
            .flat_map(|s| s.to_le_bytes())
            .collect(),
        _ => {
            return Err(format!(
                "unsupported sample format: {:?} {}bit",
                spec.sample_format, spec.bits_per_sample
            )
            .into())
        }
    };

    let mut params = default_parameters();
    params.sampleFormatInp = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
    params.sampleFormatOut = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
    params.sampleRateInp = spec.sample_rate as f32;

    let ggwave = GgWave::new(params)?;

    match ggwave.decode(&waveform)? {
        Some(payload) => {
            let text = String::from_utf8_lossy(&payload);
            println!("{text}");
        }
        None => {
            println!("No payload decoded.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_wav_path() -> PathBuf {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!("ggwave_cli_test_{}_{}.wav", std::process::id(), id));
        path
    }

    #[test]
    fn test_encode_decode_wav_roundtrip() {
        let message = "hello";
        let wav_path = temp_wav_path();

        // Encode
        encode(&message, &wav_path, 25, Protocol::AudibleFast).expect("encode failed");

        // Verify file exists
        assert!(wav_path.exists(), "WAV file should exist");

        // Decode by reading the file and checking output
        let mut reader = WavReader::open(&wav_path).expect("open wav failed");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.bits_per_sample, 32);

        let waveform: Vec<u8> = reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .expect("read samples failed")
            .into_iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();

        let mut params = default_parameters();
        params.sampleFormatInp = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
        params.sampleFormatOut = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
        params.sampleRateInp = spec.sample_rate as f32;

        let ggwave = GgWave::new(params).expect("ggwave init failed");
        let decoded = ggwave.decode(&waveform).expect("decode failed");
        let decoded = decoded.expect("no payload decoded");

        assert_eq!(decoded, b"hello");

        // Cleanup
        let _ = std::fs::remove_file(&wav_path);
    }

    #[test]
    fn test_decode_nonexistent_file() {
        let result = decode(&PathBuf::from("/nonexistent/path.wav"));
        assert!(result.is_err());
    }
}
