use super::*;

use acme_kit::text;

/// Where a usage metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage metric as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetric {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub active: bool,
    pub attempts: i64,
    pub contact_email: String,
    pub sequence: i64,
    pub principal: String,
    pub capacity: i64,
    pub status: UsageMetricStatus,
}

impl UsageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageMetricStatus) -> bool {
        match self.status {
            UsageMetricStatus::Proposed => {
                return next == UsageMetricStatus::Approved;
            }
            UsageMetricStatus::Approved => {
                return next == UsageMetricStatus::Withdrawn;
            }
            UsageMetricStatus::Rejected => {
                return next == UsageMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageMetric {
        UsageMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            active: self.active,
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            principal: self.principal.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a usage metric in status proposed.
pub fn new_usage_metric(code: &str, name: &str) -> UsageMetric {
    UsageMetric {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        active: false,
        attempts: 0,
        contact_email: "".to_string(),
        sequence: 0,
        principal: "".to_string(),
        capacity: 0,
        status: UsageMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_metric("code-885", "name-303");
        assert_eq!(item.status, UsageMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_usage_metric("code-885", "name-303");
        if item.move_to(UsageMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_metric("code-885", "name-303");
        item.status = UsageMetricStatus::Withdrawn;
        assert!(
            item.move_to(UsageMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
