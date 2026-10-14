use super::*;

use acme_kit::text;

/// Where a measurement override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MeasurementOverrideStatus {
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

impl std::fmt::Display for MeasurementOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverride {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub tenant_id: String,
    pub port: i64,
    pub window_days: i64,
    pub status: MeasurementOverrideStatus,
}

impl MeasurementOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementOverrideStatus) -> bool {
        match self.status {
            MeasurementOverrideStatus::Open => {
                match next {
                    MeasurementOverrideStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementOverrideStatus::InReview => {
                return next == MeasurementOverrideStatus::Archived;
            }
            MeasurementOverrideStatus::Resolved => {
                match next {
                    MeasurementOverrideStatus::Reopened | MeasurementOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementOverrideStatus::Reopened => {
                return next == MeasurementOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "measurement override", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementOverride {
        MeasurementOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a measurement override in status open.
pub fn new_measurement_override(code: &str, label: &str) -> MeasurementOverride {
    MeasurementOverride {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        tenant_id: "".to_string(),
        port: 0,
        window_days: 0,
        status: MeasurementOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_measurement_override("code-591", "label-839");
        assert_eq!(item.status, MeasurementOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_measurement_override("code-591", "label-839");
        if item.move_to(MeasurementOverrideStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementOverrideStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_measurement_override("code-591", "label-839");
        item.status = MeasurementOverrideStatus::Archived;
        assert!(
            item.move_to(MeasurementOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
