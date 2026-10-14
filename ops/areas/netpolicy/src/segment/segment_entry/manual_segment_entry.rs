use super::*;

use acme_kit::text;

/// Where a manual segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSegmentEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualSegmentEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ManualSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual segment entry as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentEntry {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub segment: String,
    pub visible: bool,
    pub currency: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub status: ManualSegmentEntryStatus,
}

impl ManualSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSegmentEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSegmentEntryStatus) -> bool {
        match self.status {
            ManualSegmentEntryStatus::Planned => {
                match next {
                    ManualSegmentEntryStatus::Scheduled | ManualSegmentEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSegmentEntryStatus::Scheduled => {
                return next == ManualSegmentEntryStatus::InProgress || next == ManualSegmentEntryStatus::Completed;
            }
            ManualSegmentEntryStatus::InProgress => {
                match next {
                    ManualSegmentEntryStatus::Completed | ManualSegmentEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSegmentEntryStatus::Completed => {
                return next == ManualSegmentEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual segment entry", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.version as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSegmentEntry {
        ManualSegmentEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            segment: self.segment.clone(),
            visible: self.visible,
            currency: self.currency.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual segment entry in status planned.
pub fn new_manual_segment_entry(code: &str, name: &str) -> ManualSegmentEntry {
    ManualSegmentEntry {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        segment: "".to_string(),
        visible: false,
        currency: "".to_string(),
        starts_at: 0,
        ceiling: 0.0,
        status: ManualSegmentEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_segment_entry("code-186", "name-224");
        assert_eq!(item.status, ManualSegmentEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_manual_segment_entry("code-186", "name-224");
        if item.move_to(ManualSegmentEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSegmentEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_segment_entry("code-186", "name-224");
        item.status = ManualSegmentEntryStatus::Cancelled;
        assert!(
            item.move_to(ManualSegmentEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
