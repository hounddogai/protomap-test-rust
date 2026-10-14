use super::*;

use acme_kit::text;

/// Where a manual measurement entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMeasurementEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualMeasurementEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualMeasurementEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual measurement entry as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntry {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub source: String,
    pub priority: i64,
    pub updated_at: i64,
    pub owner_id: String,
    pub tags: Vec<String>,
    pub segment: String,
    pub sequence: i64,
    pub status: ManualMeasurementEntryStatus,
}

impl ManualMeasurementEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMeasurementEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMeasurementEntryStatus) -> bool {
        match self.status {
            ManualMeasurementEntryStatus::Proposed => {
                return next == ManualMeasurementEntryStatus::Rejected;
            }
            ManualMeasurementEntryStatus::Approved => {
                match next {
                    ManualMeasurementEntryStatus::Rejected | ManualMeasurementEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMeasurementEntryStatus::Rejected => {
                return next == ManualMeasurementEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualMeasurementEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual measurement entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMeasurementEntry {
        ManualMeasurementEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            priority: self.priority,
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            tags: self.tags.clone(),
            segment: self.segment.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual measurement entry in status proposed.
pub fn new_manual_measurement_entry(code: &str, title: &str) -> ManualMeasurementEntry {
    ManualMeasurementEntry {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        source: "".to_string(),
        priority: 0,
        updated_at: 0,
        owner_id: "".to_string(),
        tags: Vec::new(),
        segment: "".to_string(),
        sequence: 0,
        status: ManualMeasurementEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_measurement_entry("code-290", "title-444");
        assert_eq!(item.status, ManualMeasurementEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_manual_measurement_entry("code-290", "title-444");
        if item.move_to(ManualMeasurementEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMeasurementEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_measurement_entry("code-290", "title-444");
        item.status = ManualMeasurementEntryStatus::Withdrawn;
        assert!(
            item.move_to(ManualMeasurementEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
