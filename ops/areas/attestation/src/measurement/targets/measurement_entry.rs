use super::*;

use acme_kit::text;

/// Where a measurement entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MeasurementEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement entry as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntry {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub duration_minutes: i64,
    pub amount: f64,
    pub pinned: bool,
    pub quota_units: i64,
    pub status: MeasurementEntryStatus,
}

impl MeasurementEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementEntryStatus) -> bool {
        match self.status {
            MeasurementEntryStatus::New => {
                match next {
                    MeasurementEntryStatus::Done | MeasurementEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementEntryStatus::Queued => {
                return next == MeasurementEntryStatus::Done;
            }
            MeasurementEntryStatus::Running => {
                return next == MeasurementEntryStatus::Failed;
            }
            MeasurementEntryStatus::Done => {
                return next == MeasurementEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementEntry {
        MeasurementEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            active: self.active,
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            pinned: self.pinned,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a measurement entry in status new.
pub fn new_measurement_entry(code: &str, name: &str) -> MeasurementEntry {
    MeasurementEntry {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        active: false,
        duration_minutes: 0,
        amount: 0.0,
        pinned: false,
        quota_units: 0,
        status: MeasurementEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_entry("code-941", "name-829");
        assert_eq!(item.status, MeasurementEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_measurement_entry("code-941", "name-829");
        if item.move_to(MeasurementEntryStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementEntryStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_entry("code-941", "name-829");
        item.status = MeasurementEntryStatus::Failed;
        assert!(
            item.move_to(MeasurementEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
