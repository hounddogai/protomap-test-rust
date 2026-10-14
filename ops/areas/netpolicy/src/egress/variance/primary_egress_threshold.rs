use super::*;

use acme_kit::text;

/// Where a primary egress threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEgressThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryEgressThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PrimaryEgressThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary egress threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressThreshold {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub priority: i64,
    pub fingerprint: String,
    pub retries: i64,
    pub attempts: i64,
    pub status: PrimaryEgressThresholdStatus,
}

impl PrimaryEgressThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEgressThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEgressThresholdStatus) -> bool {
        match self.status {
            PrimaryEgressThresholdStatus::Draft => {
                match next {
                    PrimaryEgressThresholdStatus::Pending | PrimaryEgressThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressThresholdStatus::Pending => {
                match next {
                    PrimaryEgressThresholdStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressThresholdStatus::Active => {
                return next == PrimaryEgressThresholdStatus::Suspended || next == PrimaryEgressThresholdStatus::Closed;
            }
            PrimaryEgressThresholdStatus::Suspended => {
                return next == PrimaryEgressThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEgressThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary egress threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEgressThreshold {
        PrimaryEgressThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            retries: self.retries,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary egress threshold in status draft.
pub fn new_primary_egress_threshold(code: &str, label: &str) -> PrimaryEgressThreshold {
    PrimaryEgressThreshold {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        priority: 0,
        fingerprint: "".to_string(),
        retries: 0,
        attempts: 0,
        status: PrimaryEgressThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_egress_threshold("code-508", "label-407");
        assert_eq!(item.status, PrimaryEgressThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_egress_threshold("code-508", "label-407");
        if item.move_to(PrimaryEgressThresholdStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEgressThresholdStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_egress_threshold("code-508", "label-407");
        item.status = PrimaryEgressThresholdStatus::Closed;
        assert!(
            item.move_to(PrimaryEgressThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
