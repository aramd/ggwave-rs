#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use libc::{c_float, c_int, c_void};

pub const GGWAVE_MAX_INSTANCES: c_int = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ggwave_SampleFormat {
    GGWAVE_SAMPLE_FORMAT_UNDEFINED = 0,
    GGWAVE_SAMPLE_FORMAT_U8 = 1,
    GGWAVE_SAMPLE_FORMAT_I8 = 2,
    GGWAVE_SAMPLE_FORMAT_U16 = 3,
    GGWAVE_SAMPLE_FORMAT_I16 = 4,
    GGWAVE_SAMPLE_FORMAT_F32 = 5,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ggwave_ProtocolId {
    GGWAVE_PROTOCOL_AUDIBLE_NORMAL = 0,
    GGWAVE_PROTOCOL_AUDIBLE_FAST = 1,
    GGWAVE_PROTOCOL_AUDIBLE_FASTEST = 2,
    GGWAVE_PROTOCOL_ULTRASOUND_NORMAL = 3,
    GGWAVE_PROTOCOL_ULTRASOUND_FAST = 4,
    GGWAVE_PROTOCOL_ULTRASOUND_FASTEST = 5,
    GGWAVE_PROTOCOL_DT_NORMAL = 6,
    GGWAVE_PROTOCOL_DT_FAST = 7,
    GGWAVE_PROTOCOL_DT_FASTEST = 8,
    GGWAVE_PROTOCOL_MT_NORMAL = 9,
    GGWAVE_PROTOCOL_MT_FAST = 10,
    GGWAVE_PROTOCOL_MT_FASTEST = 11,
    GGWAVE_PROTOCOL_CUSTOM_0 = 12,
    GGWAVE_PROTOCOL_CUSTOM_1 = 13,
    GGWAVE_PROTOCOL_CUSTOM_2 = 14,
    GGWAVE_PROTOCOL_CUSTOM_3 = 15,
    GGWAVE_PROTOCOL_CUSTOM_4 = 16,
    GGWAVE_PROTOCOL_CUSTOM_5 = 17,
    GGWAVE_PROTOCOL_CUSTOM_6 = 18,
    GGWAVE_PROTOCOL_CUSTOM_7 = 19,
    GGWAVE_PROTOCOL_CUSTOM_8 = 20,
    GGWAVE_PROTOCOL_CUSTOM_9 = 21,
    GGWAVE_PROTOCOL_COUNT = 22,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ggwave_Filter {
    GGWAVE_FILTER_HANN = 0,
    GGWAVE_FILTER_HAMMING = 1,
    GGWAVE_FILTER_FIRST_ORDER_HIGH_PASS = 2,
}

pub const GGWAVE_OPERATING_MODE_RX: c_int = 1 << 1;
pub const GGWAVE_OPERATING_MODE_TX: c_int = 1 << 2;
pub const GGWAVE_OPERATING_MODE_RX_AND_TX: c_int =
    GGWAVE_OPERATING_MODE_RX | GGWAVE_OPERATING_MODE_TX;
pub const GGWAVE_OPERATING_MODE_TX_ONLY_TONES: c_int = 1 << 3;
pub const GGWAVE_OPERATING_MODE_USE_DSS: c_int = 1 << 4;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ggwave_Parameters {
    pub payloadLength: c_int,
    pub sampleRateInp: c_float,
    pub sampleRateOut: c_float,
    pub sampleRate: c_float,
    pub samplesPerFrame: c_int,
    pub soundMarkerThreshold: c_float,
    pub sampleFormatInp: ggwave_SampleFormat,
    pub sampleFormatOut: ggwave_SampleFormat,
    pub operatingMode: c_int,
}

pub type ggwave_Instance = c_int;

extern "C" {
    pub fn ggwave_setLogFile(fptr: *mut c_void);
    pub fn ggwave_getDefaultParameters() -> ggwave_Parameters;
    pub fn ggwave_init(parameters: ggwave_Parameters) -> ggwave_Instance;
    pub fn ggwave_free(instance: ggwave_Instance);
    pub fn ggwave_encode(
        instance: ggwave_Instance,
        payloadBuffer: *const c_void,
        payloadSize: c_int,
        protocolId: ggwave_ProtocolId,
        volume: c_int,
        waveformBuffer: *mut c_void,
        query: c_int,
    ) -> c_int;
    pub fn ggwave_decode(
        instance: ggwave_Instance,
        waveformBuffer: *const c_void,
        waveformSize: c_int,
        payloadBuffer: *mut c_void,
    ) -> c_int;
    pub fn ggwave_ndecode(
        instance: ggwave_Instance,
        waveformBuffer: *const c_void,
        waveformSize: c_int,
        payloadBuffer: *mut c_void,
        payloadSize: c_int,
    ) -> c_int;
    pub fn ggwave_rxToggleProtocol(protocolId: ggwave_ProtocolId, state: c_int);
    pub fn ggwave_txToggleProtocol(protocolId: ggwave_ProtocolId, state: c_int);
    pub fn ggwave_rxProtocolSetFreqStart(protocolId: ggwave_ProtocolId, freqStart: c_int);
    pub fn ggwave_txProtocolSetFreqStart(protocolId: ggwave_ProtocolId, freqStart: c_int);
    pub fn ggwave_rxDurationFrames(instance: ggwave_Instance) -> c_int;
}

