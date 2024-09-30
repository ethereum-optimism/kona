//! Metrics for derivation pipeline stages.

use alloc::{boxed::Box, string::String};
use lazy_static::lazy_static;
use prometheus::{
    self, opts, register_counter_vec, register_gauge_vec, register_histogram,
    register_histogram_vec, register_int_gauge, CounterVec, GaugeVec, Histogram, HistogramVec,
    IntGauge,
};

const RESPONSE_TIME_CUSTOM_BUCKETS: &[f64; 18] = &[
    0.00001, 0.00005, 0.0001, 0.0002, 0.0005, 0.001, 0.002, 0.005, 0.008, 0.01, 0.02, 0.05, 0.08,
    0.1, 0.2, 0.5, 0.8, 1.0,
];

const FRAME_COUNT_BUCKETS: &[f64; 10] = &[1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 12.0, 15.0, 18.0, 20.0];

lazy_static! {
    /// Tracks stage resets.
    pub static ref STAGE_RESETS: GaugeVec = {
        let opts = opts!("kona_derive_stage_resets", "Number of times various stages are reset");
        register_gauge_vec!(opts, &["stage"]).expect("Stage reset metric failed to register")
    };

    /// Tracks the L1 origin for the L1 Traversal Stage.
    pub static ref ORIGIN_GAUGE: IntGauge = register_int_gauge!(
        "kona_derive_origin_gauge",
        "Tracks the L1 origin for the L1 Traversal Stage"
    ).expect("Origin Gauge failed to register");

    /// Tracks the number of frames in the current channel.
    pub static ref CURRENT_CHANNEL_FRAMES: IntGauge = register_int_gauge!(
        "kona_derive_current_channel_frames",
        "Tracks the number of frames in the current channel."
    ).expect("Current channel frames failed to register");

    /// Tracks the number of channels that have the bucket amount of frames.
    pub static ref CHANNEL_FRAME_COUNT: Histogram = register_histogram!(
        "kona_derive_channel_frame_count",
        "Tracks the number of channels that have the bucket amount of frames.",
        FRAME_COUNT_BUCKETS.to_vec()
    ).expect("Channel frame count failed to register");

    /// Tracks batch reader errors.
    pub static ref BATCH_READER_ERRORS: CounterVec = register_counter_vec!(
        "kona_derive_batch_reader_errors",
        "Number of batch reader errors",
        &["error"]
    ).expect("Batch Reader Errors failed to register");

    /// Tracks the number of times the channel queue was detected
    /// non-empty during a frame ingestion, and new channel creation
    /// was attempted post-holocene.
    pub static ref CHANNEL_QUEUE_NON_EMPTY: IntGauge = register_int_gauge!(
        "kona_derive_channel_queue_non_empty",
        "Number of times a channel was attempted to be created in the channel bank, but the queue is non-empty post-holocene."
    ).expect("Channel Queue Non Empty failed to register");

    /// Tracks the compression ratio of batches.
    pub static ref BATCH_COMPRESSION_RATIO: IntGauge = register_int_gauge!(
        "kona_derive_batch_compression_ratio",
        "Compression ratio of batches"
    ).expect("Batch Compression Ratio failed to register");

    /// Tracks the time taken for stage advance methods.
    pub static ref STAGE_ADVANCE_RESPONSE_TIME: HistogramVec = register_histogram_vec!(
        "kona_derive_stage_advance_response_time_seconds",
        "Stage advance response times",
        &["stage"],
        RESPONSE_TIME_CUSTOM_BUCKETS.to_vec()
    ).expect("Failed to register histogram vec");

    /// Tracks the number of derived frames.
    pub static ref DERIVED_FRAMES_COUNT: GaugeVec = {
        let opts = opts!("kona_derive_derived_frames_count", "Number of derived frames");
        register_gauge_vec!(opts, &["status"]).expect("Derived Frames Count failed to register")
    };

    /// Tracks the number of channel timeouts.
    pub static ref CHANNEL_TIMEOUTS: Histogram = {
        let channel_timeout_buckets: [f64; 100] = core::array::from_fn(|i| (i * 10) as f64);
        register_histogram!(
            "kona_derive_channel_timeouts",
            "Channel timeouts",
            channel_timeout_buckets.to_vec()
        ).expect("Failed to register histogram vec")
    };
}
