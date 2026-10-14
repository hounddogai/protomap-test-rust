use super::*;

use acme_kit::text;

/// Where a health check record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl HealthCheckRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for HealthCheckRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecord {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub cost: f64,
    pub margin: f64,
    pub burst: i64,
    pub unit: String,
    pub status: HealthCheckRecordStatus,
}

impl HealthCheckRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckRecordStatus) -> bool {
        match self.status {
            HealthCheckRecordStatus::Proposed => {
                return next == HealthCheckRecordStatus::Approved;
            }
            HealthCheckRecordStatus::Approved => {
                return next == HealthCheckRecordStatus::Rejected || next == HealthCheckRecordStatus::Withdrawn;
            }
            HealthCheckRecordStatus::Rejected => {
                match next {
                    HealthCheckRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.margin * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckRecord {
        HealthCheckRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            cost: self.cost,
            margin: self.margin,
            burst: self.burst,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a health check record in status proposed.
pub fn new_health_check_record(code: &str, name: &str) -> HealthCheckRecord {
    HealthCheckRecord {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        cost: 0.0,
        margin: 0.0,
        burst: 0,
        unit: "".to_string(),
        status: HealthCheckRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_health_check_record("code-774", "name-322");
        assert_eq!(item.status, HealthCheckRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_health_check_record("code-774", "name-322");
        if item.move_to(HealthCheckRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_health_check_record("code-774", "name-322");
        item.status = HealthCheckRecordStatus::Withdrawn;
        assert!(
            item.move_to(HealthCheckRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
