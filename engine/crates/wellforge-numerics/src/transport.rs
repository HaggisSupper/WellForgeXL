//! Conservative one-dimensional parcel displacement bookkeeping.

/// One labeled material parcel in a fixed-volume path.
#[derive(Clone, Debug, PartialEq)]
pub struct Parcel {
    /// Caller-provided parcel identity.
    pub label: String,
    /// Parcel volume in the caller's consistent volume unit.
    pub volume: f64,
}

/// Parcel queue construction or displacement failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParcelError {
    /// Capacity or parcel volume is non-finite or non-positive.
    InvalidVolume,
    /// A fill operation would exceed capacity or was requested on a non-empty path.
    InvalidFill,
}

/// Ordered fixed-capacity parcel queue; index zero is the outlet side.
#[derive(Clone, Debug, PartialEq)]
pub struct ParcelQueue {
    capacity: f64,
    parcels: Vec<Parcel>,
}

impl ParcelQueue {
    /// Creates an empty path with fixed positive capacity.
    ///
    /// # Errors
    /// Returns [`ParcelError::InvalidVolume`] for a non-finite or non-positive capacity.
    pub fn new(capacity: f64) -> Result<Self, ParcelError> {
        if !capacity.is_finite() || capacity <= 0.0 {
            return Err(ParcelError::InvalidVolume);
        }
        Ok(Self {
            capacity,
            parcels: Vec::new(),
        })
    }

    /// Fills an empty path with one parcel without displacement.
    ///
    /// # Errors
    /// Returns [`ParcelError`] when the volume is invalid, exceeds capacity, or the path is non-empty.
    pub fn fill(&mut self, label: impl Into<String>, volume: f64) -> Result<(), ParcelError> {
        if !volume.is_finite() || volume <= 0.0 {
            return Err(ParcelError::InvalidVolume);
        }
        if !self.parcels.is_empty() || volume > self.capacity {
            return Err(ParcelError::InvalidFill);
        }
        self.parcels.push(Parcel {
            label: label.into(),
            volume,
        });
        Ok(())
    }

    /// Injects a parcel at the inlet and returns material displaced from the outlet.
    ///
    /// The retained path volume is capped at capacity. Outlet parcels are removed or split in
    /// FIFO order so retained plus exited volume exactly equals pre-injection plus injected volume.
    ///
    /// # Errors
    /// Returns [`ParcelError::InvalidVolume`] for a non-finite or non-positive injection.
    pub fn inject(
        &mut self,
        label: impl Into<String>,
        volume: f64,
    ) -> Result<Vec<Parcel>, ParcelError> {
        if !volume.is_finite() || volume <= 0.0 {
            return Err(ParcelError::InvalidVolume);
        }
        self.parcels.push(Parcel {
            label: label.into(),
            volume,
        });

        let mut excess = (self.total_volume() - self.capacity).max(0.0);
        let mut exited = Vec::new();
        while excess > 0.0 && !self.parcels.is_empty() {
            let available = self.parcels[0].volume;
            let removed = excess.min(available);
            let label = self.parcels[0].label.clone();
            exited.push(Parcel {
                label,
                volume: removed,
            });
            self.parcels[0].volume -= removed;
            excess -= removed;
            if self.parcels[0].volume <= f64::EPSILON * self.capacity.max(1.0) {
                self.parcels.remove(0);
            }
        }
        Ok(exited)
    }

    /// Returns the current retained parcel volume.
    #[must_use]
    pub fn total_volume(&self) -> f64 {
        self.parcels.iter().map(|parcel| parcel.volume).sum()
    }

    /// Returns retained parcels in outlet-to-inlet order.
    #[must_use]
    pub fn parcels(&self) -> &[Parcel] {
        &self.parcels
    }
}
