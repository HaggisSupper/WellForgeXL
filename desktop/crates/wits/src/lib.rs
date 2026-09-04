//! Offline WITS record parsing, deterministic rig-state classification, and
//! in-memory replay journaling.
//!
//! # Scope
//!
//! This crate handles individual offline `MNEMONIC=VALUE` records and derives a
//! state from the available channels. Its replay journal retains accepted,
//! canonical records in memory only; it does not implement a streaming protocol
//! or persistence layer.
//!
//! Journal events preserve all canonical parsed mnemonics and their source and
//! UTC timing provenance. Appending equivalent content for an existing event id
//! is idempotent; different content for that id is rejected. Replay always sorts
//! by acquisition time and then event id, independently of ingest time.
//! A repeated delivery may carry a different ingest time without changing the
//! canonical event; the first accepted event remains stored.
//!
//! Mnemonics are trimmed, converted to ASCII uppercase, and must contain only
//! ASCII letters, digits, or underscores. Blank lines are ignored. If a record
//! repeats a mnemonic after canonicalization, the last occurrence wins.
//!
//! Active states require the corresponding observed channels. `Static` requires
//! at least one observed classifier channel with no active-state match; missing
//! classifier telemetry produces `Unknown`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

/// Immutable source and timing metadata for a journaled record.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RecordProvenance {
    source_id: String,
    #[serde(with = "time::serde::rfc3339")]
    acquisition_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    ingest_time: OffsetDateTime,
}

impl RecordProvenance {
    /// Identifies the source that produced the record.
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// UTC time at which the record was acquired.
    pub fn acquisition_time(&self) -> OffsetDateTime {
        self.acquisition_time
    }

    /// UTC time at which the record entered this process.
    pub fn ingest_time(&self) -> OffsetDateTime {
        self.ingest_time
    }
}

/// An immutable, canonical parsed record accepted by a [`Journal`].
///
/// Values retain every parsed mnemonic, including mnemonics that are not used
/// by rig-state classification.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct JournalEvent {
    event_id: String,
    provenance: RecordProvenance,
    values: BTreeMap<String, f64>,
}

impl JournalEvent {
    /// Creates an event from the existing portable `MNEMONIC=VALUE` input
    /// format. Timestamps must be RFC3339 instants with a UTC offset.
    pub fn from_key_value(
        event_id: impl Into<String>,
        source_id: impl Into<String>,
        acquisition_time: &str,
        ingest_time: &str,
        record: &str,
    ) -> Result<Self, JournalError> {
        let event_id = event_id.into();
        if event_id.trim().is_empty() {
            return Err(JournalError::BlankEventId);
        }

        let source_id = source_id.into();
        if source_id.trim().is_empty() {
            return Err(JournalError::BlankSourceId);
        }

        Ok(Self {
            event_id,
            provenance: RecordProvenance {
                source_id,
                acquisition_time: parse_utc_rfc3339(
                    JournalTimestampField::Acquisition,
                    acquisition_time,
                )?,
                ingest_time: parse_utc_rfc3339(JournalTimestampField::Ingest, ingest_time)?,
            },
            values: parse_key_value_record(record).map_err(JournalError::InvalidValues)?,
        })
    }

    /// Caller-supplied identity used for idempotent append handling.
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// Source and timing metadata for this event.
    pub fn provenance(&self) -> &RecordProvenance {
        &self.provenance
    }

    /// Canonical parsed mnemonic values, sorted by mnemonic.
    pub fn values(&self) -> &BTreeMap<String, f64> {
        &self.values
    }

    fn is_semantically_equivalent_to(&self, other: &Self) -> bool {
        self.event_id == other.event_id
            && self.provenance.source_id == other.provenance.source_id
            && self.provenance.acquisition_time == other.provenance.acquisition_time
            && self.values == other.values
    }
}

/// Identifies the timestamp rejected while creating a journal event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalTimestampField {
    Acquisition,
    Ingest,
}

impl std::fmt::Display for JournalTimestampField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Acquisition => "acquisition",
            Self::Ingest => "ingest",
        };
        formatter.write_str(name)
    }
}

/// Typed errors returned when constructing or appending journal events.
#[derive(Debug, Error)]
pub enum JournalError {
    #[error("journal event id must not be blank")]
    BlankEventId,
    #[error("journal source id must not be blank")]
    BlankSourceId,
    #[error("journal {field} timestamp must be an RFC3339 UTC instant: {value}")]
    InvalidTimestamp {
        field: JournalTimestampField,
        value: String,
    },
    #[error("journal event values are invalid: {0}")]
    InvalidValues(#[source] WitsError),
    #[error("journal event id already refers to different content: {event_id}")]
    ConflictingEventId { event_id: String },
}

fn parse_utc_rfc3339(
    field: JournalTimestampField,
    timestamp: &str,
) -> Result<OffsetDateTime, JournalError> {
    if timestamp.ends_with("-00:00") {
        return Err(JournalError::InvalidTimestamp {
            field,
            value: timestamp.to_owned(),
        });
    }
    let timestamp_value = timestamp.to_owned();
    let parsed =
        OffsetDateTime::parse(timestamp, &Rfc3339).map_err(|_| JournalError::InvalidTimestamp {
            field,
            value: timestamp_value,
        })?;
    if parsed.offset() != UtcOffset::UTC {
        return Err(JournalError::InvalidTimestamp {
            field,
            value: timestamp.to_owned(),
        });
    }
    Ok(parsed)
}

/// Result of attempting to append an event, always retaining the stored event.
///
/// The returned event allows callers to continue their processing without
/// re-reading the journal or dropping accepted canonical values.
#[derive(Clone, Debug, PartialEq)]
pub enum JournalAppendOutcome {
    Appended(JournalEvent),
    AlreadyPresent(JournalEvent),
}

/// In-memory, deterministic replay journal keyed by event id.
#[derive(Clone, Debug, Default)]
pub struct Journal {
    events_by_id: BTreeMap<String, JournalEvent>,
}

impl Journal {
    /// Appends a new event or reports its canonical stored equivalent.
    ///
    /// An event id can never silently replace existing content: equivalent
    /// duplicates are idempotent and conflicting duplicates are rejected. A
    /// redelivery may have a different ingest time, but retains the first
    /// accepted event rather than replacing it.
    pub fn append(&mut self, event: JournalEvent) -> Result<JournalAppendOutcome, JournalError> {
        if let Some(existing) = self.events_by_id.get(event.event_id()) {
            return if existing.is_semantically_equivalent_to(&event) {
                Ok(JournalAppendOutcome::AlreadyPresent(existing.clone()))
            } else {
                Err(JournalError::ConflictingEventId {
                    event_id: event.event_id().to_owned(),
                })
            };
        }

        self.events_by_id
            .insert(event.event_id().to_owned(), event.clone());
        Ok(JournalAppendOutcome::Appended(event))
    }

    /// Returns every stored event in deterministic replay order: acquisition
    /// time first, then event id. Ingest time does not affect replay order.
    pub fn replay(&self) -> Vec<JournalEvent> {
        let mut events = self.events_by_id.values().cloned().collect::<Vec<_>>();
        events.sort_by(|left, right| {
            left.provenance
                .acquisition_time
                .cmp(&right.provenance.acquisition_time)
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        events
    }

    /// Number of distinct event ids retained by this journal.
    pub fn len(&self) -> usize {
        self.events_by_id.len()
    }

    /// Returns whether no events have been retained.
    pub fn is_empty(&self) -> bool {
        self.events_by_id.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RigChannels {
    pub wob_kn: Option<f64>,
    pub rpm: Option<f64>,
    pub flow_lpm: Option<f64>,
    pub hookload_kn: Option<f64>,
    pub standpipe_pressure_kpa: Option<f64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RigState {
    RotaryDrilling,
    Sliding,
    Circulating,
    Tripping,
    Static,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct RigStateThresholds {
    pub on_bottom_wob_kn: f64,
    pub rotating_rpm: f64,
    pub circulating_flow_lpm: f64,
}
impl Default for RigStateThresholds {
    fn default() -> Self {
        Self {
            on_bottom_wob_kn: 5.0,
            rotating_rpm: 5.0,
            circulating_flow_lpm: 50.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum WitsError {
    #[error("invalid WITS record at line {line}")]
    InvalidRecord { line: usize },
    #[error("WITS channel value is not finite")]
    NonFiniteValue,
    #[error("rig-state thresholds are invalid")]
    InvalidThresholds,
}

/// Parses a portable offline `MNEMONIC=VALUE` WITS text record. Unknown
/// mnemonics are retained by the caller's source reader, not discarded here.
pub fn parse_key_value_record(input: &str) -> Result<BTreeMap<String, f64>, WitsError> {
    let mut fields = BTreeMap::new();
    for (index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(WitsError::InvalidRecord { line: index + 1 });
        };
        let key = canonical_mnemonic(key).ok_or(WitsError::InvalidRecord { line: index + 1 })?;
        let value = value
            .trim()
            .parse::<f64>()
            .map_err(|_| WitsError::InvalidRecord { line: index + 1 })?;
        if !value.is_finite() {
            return Err(WitsError::NonFiniteValue);
        }
        fields.insert(key, value);
    }
    Ok(fields)
}

fn canonical_mnemonic(key: &str) -> Option<String> {
    let key = key.trim();
    if key.is_empty()
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    Some(key.to_ascii_uppercase())
}

pub fn channels_from_record(record: &BTreeMap<String, f64>) -> RigChannels {
    let take = |keys: &[&str]| keys.iter().find_map(|key| record.get(*key).copied());
    RigChannels {
        wob_kn: take(&["WOB_KN", "WOB"]),
        rpm: take(&["RPM"]),
        flow_lpm: take(&["FLOW_LPM", "GPM"]),
        hookload_kn: take(&["HOOKLOAD_KN", "HKLD"]),
        standpipe_pressure_kpa: take(&["SPP_KPA", "SPP"]),
    }
}

pub fn classify_rig_state(
    channels: &RigChannels,
    thresholds: RigStateThresholds,
) -> Result<RigState, WitsError> {
    if ![
        thresholds.on_bottom_wob_kn,
        thresholds.rotating_rpm,
        thresholds.circulating_flow_lpm,
    ]
    .iter()
    .all(|value| value.is_finite() && *value >= 0.0)
    {
        return Err(WitsError::InvalidThresholds);
    }
    if ![
        channels.wob_kn,
        channels.rpm,
        channels.flow_lpm,
        channels.hookload_kn,
        channels.standpipe_pressure_kpa,
    ]
    .iter()
    .flatten()
    .all(|value| value.is_finite())
    {
        return Err(WitsError::NonFiniteValue);
    }
    let meets = |value: Option<f64>, threshold: f64| value.is_some_and(|value| value >= threshold);
    Ok(
        if meets(channels.wob_kn, thresholds.on_bottom_wob_kn)
            && meets(channels.rpm, thresholds.rotating_rpm)
        {
            RigState::RotaryDrilling
        } else if meets(channels.wob_kn, thresholds.on_bottom_wob_kn)
            && meets(channels.flow_lpm, thresholds.circulating_flow_lpm)
        {
            RigState::Sliding
        } else if meets(channels.flow_lpm, thresholds.circulating_flow_lpm) {
            RigState::Circulating
        } else if channels.hookload_kn.is_some_and(|load| load > 0.0) {
            RigState::Tripping
        } else if channels.wob_kn.is_some()
            || channels.rpm.is_some()
            || channels.flow_lpm.is_some()
            || channels.hookload_kn.is_some()
        {
            RigState::Static
        } else {
            RigState::Unknown
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thresholds() -> RigStateThresholds {
        RigStateThresholds {
            on_bottom_wob_kn: 5.0,
            rotating_rpm: 5.0,
            circulating_flow_lpm: 50.0,
        }
    }

    #[test]
    fn parser_trims_and_canonicalizes_mnemonics() {
        let record = parse_key_value_record("  wob_kn  =  5.25  \n\tRpM\t= 120 ").unwrap();

        assert_eq!(record.get("WOB_KN"), Some(&5.25));
        assert_eq!(record.get("RPM"), Some(&120.0));
    }

    #[test]
    fn parser_uses_last_value_for_duplicate_canonical_mnemonics() {
        let record = parse_key_value_record("wob=1\n WOB = 2").unwrap();

        assert_eq!(record, BTreeMap::from([("WOB".to_owned(), 2.0)]));
    }

    #[test]
    fn parser_rejects_malformed_records() {
        for input in ["RPM 120", "=120", "WOB KN=5", "RPM=1=2"] {
            assert!(matches!(
                parse_key_value_record(input),
                Err(WitsError::InvalidRecord { .. })
            ));
        }
    }

    #[test]
    fn parser_rejects_non_finite_values() {
        for input in ["RPM=NaN", "FLOW=inf", "WOB=-Infinity"] {
            assert!(matches!(
                parse_key_value_record(input),
                Err(WitsError::NonFiniteValue)
            ));
        }
    }

    #[test]
    fn classifier_uses_inclusive_threshold_boundaries_and_precedence() {
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    wob_kn: Some(5.0),
                    rpm: Some(5.0),
                    flow_lpm: Some(50.0),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::RotaryDrilling
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    wob_kn: Some(5.0),
                    rpm: Some(4.9),
                    flow_lpm: Some(50.0),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::Sliding
        );
    }

    #[test]
    fn classifier_selects_circulating_tripping_and_static_below_boundaries() {
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    wob_kn: Some(4.9),
                    flow_lpm: Some(50.0),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::Circulating
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    flow_lpm: Some(49.9),
                    hookload_kn: Some(0.1),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::Tripping
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    hookload_kn: Some(0.0),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::Static
        );
    }

    #[test]
    fn classifier_returns_unknown_when_classifier_telemetry_is_absent() {
        assert_eq!(
            classify_rig_state(&RigChannels::default(), thresholds()).unwrap(),
            RigState::Unknown
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    standpipe_pressure_kpa: Some(0.0),
                    ..RigChannels::default()
                },
                thresholds(),
            )
            .unwrap(),
            RigState::Unknown
        );
    }

    #[test]
    fn zero_thresholds_require_observed_channels() {
        let zero_thresholds = RigStateThresholds {
            on_bottom_wob_kn: 0.0,
            rotating_rpm: 0.0,
            circulating_flow_lpm: 0.0,
        };

        assert_eq!(
            classify_rig_state(&RigChannels::default(), zero_thresholds).unwrap(),
            RigState::Unknown
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    wob_kn: Some(0.0),
                    ..RigChannels::default()
                },
                zero_thresholds,
            )
            .unwrap(),
            RigState::Static
        );
        assert_eq!(
            classify_rig_state(
                &RigChannels {
                    wob_kn: Some(0.0),
                    rpm: Some(0.0),
                    ..RigChannels::default()
                },
                zero_thresholds,
            )
            .unwrap(),
            RigState::RotaryDrilling
        );
    }

    #[test]
    fn classifier_rejects_negative_and_non_finite_thresholds() {
        for invalid_thresholds in [
            RigStateThresholds {
                on_bottom_wob_kn: -0.1,
                ..thresholds()
            },
            RigStateThresholds {
                rotating_rpm: f64::NAN,
                ..thresholds()
            },
            RigStateThresholds {
                circulating_flow_lpm: f64::INFINITY,
                ..thresholds()
            },
            RigStateThresholds {
                circulating_flow_lpm: f64::NEG_INFINITY,
                ..thresholds()
            },
        ] {
            assert!(matches!(
                classify_rig_state(&RigChannels::default(), invalid_thresholds),
                Err(WitsError::InvalidThresholds)
            ));
        }
    }

    #[test]
    fn classifier_rejects_non_finite_channel_values_including_hookload() {
        for channels in [
            RigChannels {
                wob_kn: Some(f64::NAN),
                ..RigChannels::default()
            },
            RigChannels {
                rpm: Some(f64::INFINITY),
                ..RigChannels::default()
            },
            RigChannels {
                flow_lpm: Some(f64::NEG_INFINITY),
                ..RigChannels::default()
            },
            RigChannels {
                hookload_kn: Some(f64::INFINITY),
                ..RigChannels::default()
            },
            RigChannels {
                standpipe_pressure_kpa: Some(f64::NEG_INFINITY),
                ..RigChannels::default()
            },
        ] {
            assert!(matches!(
                classify_rig_state(&channels, thresholds()),
                Err(WitsError::NonFiniteValue)
            ));
        }
    }

    #[test]
    fn journal_event_canonicalizes_parsed_values_and_preserves_provenance() {
        let event = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            " rpm = 120\ncustom_value = 3.5",
        )
        .unwrap();

        assert_eq!(event.event_id(), "event-1");
        assert_eq!(event.provenance().source_id(), "surface-reader-a");
        assert_eq!(
            event.values(),
            &BTreeMap::from([("CUSTOM_VALUE".to_owned(), 3.5), ("RPM".to_owned(), 120.0),])
        );
        assert_eq!(
            event.provenance().acquisition_time().to_string(),
            "2026-08-24 12:02:00.0 +00:00:00"
        );
    }

    #[test]
    fn journal_event_serializes_provenance_timestamps_as_rfc3339() {
        let event = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120",
        )
        .unwrap();

        let serialized = serde_json::to_value(event).unwrap();

        assert_eq!(
            serialized["provenance"]["acquisition_time"],
            serde_json::Value::String("2026-08-24T12:02:00Z".to_owned())
        );
        assert_eq!(
            serialized["provenance"]["ingest_time"],
            serde_json::Value::String("2026-08-24T12:03:00Z".to_owned())
        );
    }

    #[test]
    fn journal_append_reports_the_stored_event_without_dropping_values() {
        let event = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120\nUNKNOWN_CHANNEL=3.5",
        )
        .unwrap();

        let outcome = Journal::default().append(event).unwrap();

        match outcome {
            JournalAppendOutcome::Appended(stored) => {
                assert_eq!(stored.values().get("UNKNOWN_CHANNEL"), Some(&3.5));
            }
            JournalAppendOutcome::AlreadyPresent(_) => panic!("new event must be appended"),
        }
    }

    #[test]
    fn journal_append_is_idempotent_for_semantically_equivalent_duplicate_ids() {
        let first = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120\nWOB=5",
        )
        .unwrap();
        let duplicate = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00+00:00",
            "2026-08-24T12:03:00+00:00",
            " wob = 5.0\n rpm = 120.0 ",
        )
        .unwrap();
        let mut journal = Journal::default();

        journal.append(first).unwrap();
        let outcome = journal.append(duplicate).unwrap();

        assert!(matches!(outcome, JournalAppendOutcome::AlreadyPresent(_)));
        assert_eq!(journal.len(), 1);
    }

    #[test]
    fn journal_append_is_idempotent_when_only_delivery_ingest_time_is_later() {
        let original = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120",
        )
        .unwrap();
        let redelivery = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:04:00Z",
            "rpm=120.0",
        )
        .unwrap();
        let mut journal = Journal::default();

        journal.append(original).unwrap();
        let outcome = journal.append(redelivery).unwrap();

        match outcome {
            JournalAppendOutcome::AlreadyPresent(stored) => assert_eq!(
                stored.provenance().ingest_time().to_string(),
                "2026-08-24 12:03:00.0 +00:00:00"
            ),
            JournalAppendOutcome::Appended(_) => panic!("redelivery must be idempotent"),
        }
        assert_eq!(journal.len(), 1);
    }

    #[test]
    fn journal_append_rejects_conflicting_duplicate_ids_without_replacing_the_original() {
        let original = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120",
        )
        .unwrap();
        let conflict = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=121",
        )
        .unwrap();
        let mut journal = Journal::default();

        journal.append(original).unwrap();
        let error = journal.append(conflict).unwrap_err();

        assert!(matches!(error, JournalError::ConflictingEventId { .. }));
        assert_eq!(journal.replay()[0].values().get("RPM"), Some(&120.0));
    }

    #[test]
    fn journal_replay_orders_events_by_acquisition_time_then_event_id_not_ingest_time() {
        let later_acquisition = JournalEvent::from_key_value(
            "event-b",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=120",
        )
        .unwrap();
        let same_acquisition = JournalEvent::from_key_value(
            "event-a",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:04:00Z",
            "RPM=121",
        )
        .unwrap();
        let earlier_acquisition = JournalEvent::from_key_value(
            "event-c",
            "surface-reader-a",
            "2026-08-24T12:01:00Z",
            "2026-08-24T12:05:00Z",
            "RPM=122",
        )
        .unwrap();
        let mut journal = Journal::default();

        journal.append(later_acquisition).unwrap();
        journal.append(same_acquisition).unwrap();
        journal.append(earlier_acquisition).unwrap();

        let event_ids = journal
            .replay()
            .into_iter()
            .map(|event| event.event_id().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(event_ids, vec!["event-c", "event-a", "event-b"]);
    }

    #[test]
    fn journal_event_rejects_blank_identifiers_and_malformed_or_non_utc_timestamps() {
        for (event_id, source_id, acquisition_time, ingest_time) in [
            (
                " ",
                "source",
                "2026-08-24T12:02:00Z",
                "2026-08-24T12:03:00Z",
            ),
            (
                "event",
                "\t",
                "2026-08-24T12:02:00Z",
                "2026-08-24T12:03:00Z",
            ),
            ("event", "source", "not-a-timestamp", "2026-08-24T12:03:00Z"),
            ("event", "source", "2026-08-24T12:02:00Z", "not-a-timestamp"),
            (
                "event",
                "source",
                "2026-08-24T12:02:00-05:00",
                "2026-08-24T12:03:00Z",
            ),
            (
                "event",
                "source",
                "2026-08-24T12:02:00-00:00",
                "2026-08-24T12:03:00Z",
            ),
            (
                "event",
                "source",
                "2026-08-24T12:02:00Z",
                "2026-08-24T12:03:00-00:00",
            ),
        ] {
            assert!(
                JournalEvent::from_key_value(
                    event_id,
                    source_id,
                    acquisition_time,
                    ingest_time,
                    "RPM=120",
                )
                .is_err()
            );
        }
    }

    #[test]
    fn journal_event_rejects_invalid_parsed_values() {
        let error = JournalEvent::from_key_value(
            "event-1",
            "surface-reader-a",
            "2026-08-24T12:02:00Z",
            "2026-08-24T12:03:00Z",
            "RPM=NaN",
        )
        .unwrap_err();

        assert!(matches!(error, JournalError::InvalidValues { .. }));
    }
}
