use rodio::{
    DeviceSinkBuilder, MixerDeviceSink,
    buffer::SamplesBuffer,
    queue::{SourcesQueueInput, queue},
};
use std::{num::NonZero, sync::Arc};

/// Convenience wrapper around an audio queue
pub struct Audio {
    /// The sample queue
    queue: Arc<SourcesQueueInput>,
    // stream: Stream,
    sink: MixerDeviceSink,
}

impl Audio {
    pub fn new() -> Audio {
        let sink = DeviceSinkBuilder::from_default_device()
            .unwrap()
            .with_sample_rate(NonZero::new(32_000).unwrap())
            .with_channels(NonZero::new(1).unwrap())
            .open_stream()
            .expect("Unable to open default sink");
        let (input, output) = queue(true);
        sink.mixer().add(output);
        Audio { queue: input, sink }
    }
    /// Append a bunch of samples to the audio queue
    pub fn push_samples(&mut self, samples: &[f32], volume: f32) {
        if samples.len() == 0 {
            return;
        }
        self.queue.append(SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(32_000).unwrap(),
            samples.iter().map(|s| s * volume).collect::<Vec<f32>>(),
        ));
    }
}
