pub mod ffi;

use libc::{c_int, c_void};
use std::marker::PhantomData;
use std::rc::Rc;

pub use ffi::{
    ggwave_Parameters as Parameters, ggwave_ProtocolId as ProtocolId,
    ggwave_SampleFormat as SampleFormat, GGWAVE_OPERATING_MODE_RX,
    GGWAVE_OPERATING_MODE_RX_AND_TX, GGWAVE_OPERATING_MODE_TX,
    GGWAVE_OPERATING_MODE_TX_ONLY_TONES, GGWAVE_OPERATING_MODE_USE_DSS,
};

pub const MAX_DATA_SIZE: usize = 256;

#[derive(Debug)]
pub enum Error {
    InitFailed,
    EncodeFailed,
    DecodeFailed,
    BufferTooSmall,
    InvalidInput(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InitFailed => write!(f, "failed to initialize ggwave instance"),
            Error::EncodeFailed => write!(f, "failed to encode payload"),
            Error::DecodeFailed => write!(f, "failed to decode waveform"),
            Error::BufferTooSmall => write!(f, "payload buffer too small"),
            Error::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// A ggwave encoder/decoder instance.
///
/// # Thread Safety
///
/// `GgWave` is intentionally `!Send` and `!Sync`. The upstream C library stores
/// all instances in a global array (`g_instances[GGWAVE_MAX_INSTANCES]`) without
/// any synchronization. Concurrent access from multiple threads would cause data
/// races and undefined behavior. Create and use each instance on a single thread.
pub struct GgWave {
    instance: ffi::ggwave_Instance,
    parameters: Parameters,
    // PhantomData<Rc<()>> makes this type !Send and !Sync.
    _not_send_sync: PhantomData<Rc<()>>,
}

impl GgWave {
    pub fn new(parameters: Parameters) -> Result<Self, Error> {
        let instance = unsafe { ffi::ggwave_init(parameters) };
        if instance < 0 {
            return Err(Error::InitFailed);
        }

        Ok(Self {
            instance,
            parameters,
            _not_send_sync: PhantomData,
        })
    }

    pub fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    pub fn encode(
        &self,
        payload: &[u8],
        protocol: ProtocolId,
        volume: i32,
    ) -> Result<Vec<u8>, Error> {
        if !(0..=100).contains(&volume) {
            return Err(Error::InvalidInput("volume must be between 0 and 100"));
        }

        let payload_len = to_c_int(payload.len(), "payload too large")?;
        let size = unsafe {
            ffi::ggwave_encode(
                self.instance,
                payload.as_ptr() as *const c_void,
                payload_len,
                protocol,
                volume as c_int,
                std::ptr::null_mut(),
                1,
            )
        };

        if size <= 0 {
            return Err(Error::EncodeFailed);
        }

        let mut waveform = vec![0u8; size as usize];
        let written = unsafe {
            ffi::ggwave_encode(
                self.instance,
                payload.as_ptr() as *const c_void,
                payload_len,
                protocol,
                volume as c_int,
                waveform.as_mut_ptr() as *mut c_void,
                0,
            )
        };

        if written <= 0 {
            return Err(Error::EncodeFailed);
        }

        waveform.truncate(written as usize);
        Ok(waveform)
    }

    pub fn decode(&self, waveform: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let waveform_len = to_c_int(waveform.len(), "waveform too large")?;
        let mut payload = vec![0u8; MAX_DATA_SIZE];
        let decoded = unsafe {
            ffi::ggwave_ndecode(
                self.instance,
                waveform.as_ptr() as *const c_void,
                waveform_len,
                payload.as_mut_ptr() as *mut c_void,
                payload.len() as c_int,
            )
        };

        match decoded {
            0 => Ok(None),
            -1 => Err(Error::DecodeFailed),
            -2 => Err(Error::BufferTooSmall),
            n if n > 0 => {
                payload.truncate(n as usize);
                Ok(Some(payload))
            }
            _ => Err(Error::DecodeFailed),
        }
    }

    pub fn rx_duration_frames(&self) -> i32 {
        unsafe { ffi::ggwave_rxDurationFrames(self.instance) }
    }
}

impl Drop for GgWave {
    fn drop(&mut self) {
        unsafe { ffi::ggwave_free(self.instance) };
    }
}

pub fn default_parameters() -> Parameters {
    unsafe { ffi::ggwave_getDefaultParameters() }
}

pub fn set_rx_protocol_enabled(protocol: ProtocolId, enabled: bool) {
    unsafe { ffi::ggwave_rxToggleProtocol(protocol, if enabled { 1 } else { 0 }) };
}

pub fn set_tx_protocol_enabled(protocol: ProtocolId, enabled: bool) {
    unsafe { ffi::ggwave_txToggleProtocol(protocol, if enabled { 1 } else { 0 }) };
}

fn to_c_int(value: usize, context: &'static str) -> Result<c_int, Error> {
    c_int::try_from(value).map_err(|_| Error::InvalidInput(context))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encode_decode() {
        let params = default_parameters();
        let tx = GgWave::new(params).expect("tx init failed");
        let waveform = tx
            .encode(
                b"ping",
                ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
                25,
            )
            .expect("encode failed");

        let rx = GgWave::new(params).expect("rx init failed");
        let decoded = rx.decode(&waveform).expect("decode failed");
        let decoded = decoded.expect("no payload decoded");
        assert_eq!(decoded, b"ping");
    }
}

