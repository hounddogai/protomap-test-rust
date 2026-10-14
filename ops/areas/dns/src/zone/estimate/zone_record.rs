use super::*;

use acme_kit::text;

/// Where a zone record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneRecordStatus {
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

impl std::fmt::Display for ZoneRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone record as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecord {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub priority: i64,
    pub segment: String,
    pub burst: i64,
    pub budget: f64,
    pub sequence: i64,
    pub status: ZoneRecordStatus,
}

impl ZoneRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneRecordStatus) -> bool {
        match self.status {
            ZoneRecordStatus::Planned => {
                return next == ZoneRecordStatus::Cancelled;
            }
            ZoneRecordStatus::Scheduled => {
                return next == ZoneRecordStatus::InProgress || next == ZoneRecordStatus::Completed;
            }
            ZoneRecordStatus::InProgress => {
                return next == ZoneRecordStatus::Completed || next == ZoneRecordStatus::Cancelled;
            }
            ZoneRecordStatus::Completed => {
                return next == ZoneRecordStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneRecord {
        ZoneRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            priority: self.priority,
            segment: self.segment.clone(),
            burst: self.burst,
            budget: self.budget,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a zone record in status planned.
pub fn new_zone_record(code: &str, name: &str) -> ZoneRecord {
    ZoneRecord {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        priority: 0,
        segment: "".to_string(),
        burst: 0,
        budget: 0.0,
        sequence: 0,
        status: ZoneRecordStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_record("code-731", "name-622");
        assert_eq!(item.status, ZoneRecordStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_zone_record("code-731", "name-622");
        if item.move_to(ZoneRecordStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneRecordStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_record("code-731", "name-622");
        item.status = ZoneRecordStatus::Cancelled;
        assert!(
            item.move_to(ZoneRecordStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
