use super::*;

use acme_kit::text;

/// Where a policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyThresholdStatus {
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

impl std::fmt::Display for PolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyThreshold {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub segment: String,
    pub minimum: f64,
    pub maximum: f64,
    pub expires_at: i64,
    pub country: String,
    pub visible: bool,
    pub status: PolicyThresholdStatus,
}

impl PolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyThresholdStatus) -> bool {
        match self.status {
            PolicyThresholdStatus::Planned => {
                return next == PolicyThresholdStatus::Completed;
            }
            PolicyThresholdStatus::Scheduled => {
                match next {
                    PolicyThresholdStatus::Completed | PolicyThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyThresholdStatus::InProgress => {
                match next {
                    PolicyThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyThresholdStatus::Completed => {
                match next {
                    PolicyThresholdStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyThreshold {
        PolicyThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            segment: self.segment.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            expires_at: self.expires_at,
            country: self.country.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy threshold in status planned.
pub fn new_policy_threshold(code: &str, label: &str) -> PolicyThreshold {
    PolicyThreshold {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        segment: "".to_string(),
        minimum: 0.0,
        maximum: 0.0,
        expires_at: 0,
        country: "".to_string(),
        visible: false,
        status: PolicyThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_threshold("code-726", "label-36");
        assert_eq!(item.status, PolicyThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_policy_threshold("code-726", "label-36");
        if item.move_to(PolicyThresholdStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyThresholdStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_threshold("code-726", "label-36");
        item.status = PolicyThresholdStatus::Cancelled;
        assert!(
            item.move_to(PolicyThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
