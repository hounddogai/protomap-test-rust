use super::*;

use acme_kit::text;

/// Where a slo schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SloScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSchedule {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub segment: String,
    pub ends_at: i64,
    pub rotation_days: i64,
    pub cipher: String,
    pub checksum: String,
    pub status: SloScheduleStatus,
}

impl SloSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloScheduleStatus) -> bool {
        match self.status {
            SloScheduleStatus::Open => {
                match next {
                    SloScheduleStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloScheduleStatus::InReview => {
                match next {
                    SloScheduleStatus::Resolved | SloScheduleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloScheduleStatus::Resolved => {
                return next == SloScheduleStatus::Archived;
            }
            SloScheduleStatus::Reopened => {
                return next == SloScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloSchedule {
        SloSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            segment: self.segment.clone(),
            ends_at: self.ends_at,
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a slo schedule in status open.
pub fn new_slo_schedule(code: &str, name: &str) -> SloSchedule {
    SloSchedule {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        segment: "".to_string(),
        ends_at: 0,
        rotation_days: 0,
        cipher: "".to_string(),
        checksum: "".to_string(),
        status: SloScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_schedule("code-37", "name-898");
        assert_eq!(item.status, SloScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_slo_schedule("code-37", "name-898");
        if item.move_to(SloScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_schedule("code-37", "name-898");
        item.status = SloScheduleStatus::Archived;
        assert!(item.move_to(SloScheduleStatus::Open).is_err(), "expected move to fail");
    }
}
