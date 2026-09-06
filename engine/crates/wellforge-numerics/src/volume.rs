//! Conservative piecewise cumulative-volume coordinate.

use crate::Interval;

/// One constant-area segment in a piecewise volume coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumeSegment {
    /// Spatial interval occupied by this capacity segment.
    pub interval: Interval,
    /// Cross-sectional area in caller-defined consistent units.
    pub area: f64,
}

/// Volume-coordinate construction or lookup failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeError {
    /// No capacity segments were supplied.
    Empty,
    /// A coordinate or area is NaN or infinite.
    NonFinite,
    /// An interval has zero/negative length or an area is negative.
    InvalidSegment,
    /// Consecutive intervals are not contiguous and ordered.
    Discontinuous,
    /// A requested position or cumulative volume lies outside the coordinate.
    OutOfRange,
}

fn same_value(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits()
}

/// Deterministic mapping between a piecewise spatial coordinate and cumulative volume.
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeCoordinate {
    segments: Vec<VolumeSegment>,
    cumulative_start: Vec<f64>,
    total_volume: f64,
}

impl VolumeCoordinate {
    /// Constructs a conservative cumulative-volume coordinate.
    ///
    /// # Errors
    /// Returns [`VolumeError`] when segments are empty, non-finite, invalid, or discontinuous.
    pub fn new(segments: Vec<VolumeSegment>) -> Result<Self, VolumeError> {
        if segments.is_empty() {
            return Err(VolumeError::Empty);
        }
        let mut cumulative_start = Vec::with_capacity(segments.len());
        let mut cumulative = 0.0;
        for (index, segment) in segments.iter().enumerate() {
            if !segment.interval.start.is_finite()
                || !segment.interval.end.is_finite()
                || !segment.area.is_finite()
            {
                return Err(VolumeError::NonFinite);
            }
            if segment.interval.end <= segment.interval.start || segment.area < 0.0 {
                return Err(VolumeError::InvalidSegment);
            }
            if index > 0
                && !same_value(segment.interval.start, segments[index - 1].interval.end)
            {
                return Err(VolumeError::Discontinuous);
            }
            cumulative_start.push(cumulative);
            cumulative += (segment.interval.end - segment.interval.start) * segment.area;
            if !cumulative.is_finite() {
                return Err(VolumeError::NonFinite);
            }
        }
        Ok(Self {
            segments,
            cumulative_start,
            total_volume: cumulative,
        })
    }

    /// Returns the total capacity represented by all segments.
    #[must_use]
    pub const fn total_volume(&self) -> f64 {
        self.total_volume
    }

    /// Returns cumulative volume from the first coordinate boundary to `position`.
    ///
    /// # Errors
    /// Returns [`VolumeError::OutOfRange`] when `position` lies outside the coordinate and
    /// [`VolumeError::NonFinite`] for NaN/infinite input.
    pub fn volume_at(&self, position: f64) -> Result<f64, VolumeError> {
        if !position.is_finite() {
            return Err(VolumeError::NonFinite);
        }
        let first = self.segments[0].interval.start;
        let last = self.segments[self.segments.len() - 1].interval.end;
        if position < first || position > last {
            return Err(VolumeError::OutOfRange);
        }
        if same_value(position, last) {
            return Ok(self.total_volume);
        }
        for (index, segment) in self.segments.iter().enumerate() {
            if position >= segment.interval.start && position < segment.interval.end {
                return Ok(self.cumulative_start[index]
                    + (position - segment.interval.start) * segment.area);
            }
        }
        Err(VolumeError::OutOfRange)
    }

    /// Returns the earliest spatial position corresponding to cumulative `volume`.
    ///
    /// Zero-area segments create volume plateaus; for an exact plateau value this method returns
    /// the earliest position, making the inverse deterministic.
    ///
    /// # Errors
    /// Returns [`VolumeError::OutOfRange`] when `volume` lies outside `[0,total_volume]` and
    /// [`VolumeError::NonFinite`] for NaN/infinite input.
    pub fn position_at(&self, volume: f64) -> Result<f64, VolumeError> {
        if !volume.is_finite() {
            return Err(VolumeError::NonFinite);
        }
        if volume < 0.0 || volume > self.total_volume {
            return Err(VolumeError::OutOfRange);
        }
        for (index, segment) in self.segments.iter().enumerate() {
            let start_volume = self.cumulative_start[index];
            let segment_volume = (segment.interval.end - segment.interval.start) * segment.area;
            let end_volume = start_volume + segment_volume;
            if same_value(volume, start_volume) {
                return Ok(segment.interval.start);
            }
            if segment.area > 0.0 && volume > start_volume && volume <= end_volume {
                return Ok(segment.interval.start + (volume - start_volume) / segment.area);
            }
        }
        if same_value(volume, self.total_volume) {
            return Ok(self.segments[self.segments.len() - 1].interval.end);
        }
        Err(VolumeError::OutOfRange)
    }
}
