//! Stable JSON Lines diagnostics for detailed executable-boundary events.

use serde::Serialize;

/// One stable structured diagnostic record.
#[derive(Serialize)]
pub(crate) struct Diagnostic<'a> {
    level: &'a str,
    event: &'a str,
    code: &'a str,
    analysis_id: &'a str,
    request_hash: &'a str,
    message: &'a str,
}

impl<'a> Diagnostic<'a> {
    /// Creates an informational diagnostic with the required stable field set.
    pub(crate) fn info(
        event: &'a str,
        code: &'a str,
        analysis_id: &'a str,
        request_hash: &'a str,
        message: &'a str,
    ) -> Self {
        Self {
            level: "info",
            event,
            code,
            analysis_id,
            request_hash,
            message,
        }
    }
}

/// Serializes diagnostics as one compact JSON object per UTF-8 line.
pub(crate) fn bytes(records: &[Diagnostic<'_>]) -> Result<Vec<u8>, serde_json::Error> {
    let mut output = Vec::new();
    for record in records {
        serde_json::to_writer(&mut output, record)?;
        output.push(b'\n');
    }
    Ok(output)
}
