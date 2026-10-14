use super::*;

use acme_kit::text;

/// Where a identity metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityMetricStatus {
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

impl std::fmt::Display for IdentityMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity metric as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMetric {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub rotation_days: i64,
    pub country: String,
    pub maximum: f64,
    pub status: IdentityMetricStatus,
}

impl IdentityMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityMetricStatus) -> bool {
        match self.status {
            IdentityMetricStatus::Planned => {
                match next {
                    IdentityMetricStatus::Completed | IdentityMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityMetricStatus::Scheduled => {
                return next == IdentityMetricStatus::Completed || next == IdentityMetricStatus::Cancelled;
            }
            IdentityMetricStatus::InProgress => {
                match next {
                    IdentityMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityMetricStatus::Completed => {
                match next {
                    IdentityMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IdentityMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity metric", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.maximum - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityMetric {
        IdentityMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            rotation_days: self.rotation_days,
            country: self.country.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a identity metric in status planned.
pub fn new_identity_metric(code: &str, label: &str) -> IdentityMetric {
    IdentityMetric {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        rotation_days: 0,
        country: "".to_string(),
        maximum: 0.0,
        status: IdentityMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_identity_metric("code-109", "label-464");
        assert_eq!(item.status, IdentityMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_identity_metric("code-109", "label-464");
        if item.move_to(IdentityMetricStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityMetricStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_identity_metric("code-109", "label-464");
        item.status = IdentityMetricStatus::Cancelled;
        assert!(
            item.move_to(IdentityMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
