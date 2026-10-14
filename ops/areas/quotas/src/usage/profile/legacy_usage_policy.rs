use super::*;

use acme_kit::text;

/// Where a legacy usage policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyUsagePolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyUsagePolicyStatus {
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

impl std::fmt::Display for LegacyUsagePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy usage policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicy {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub ends_at: i64,
    pub quantity: i64,
    pub status: LegacyUsagePolicyStatus,
}

impl LegacyUsagePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyUsagePolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyUsagePolicyStatus) -> bool {
        match self.status {
            LegacyUsagePolicyStatus::Planned => {
                return next == LegacyUsagePolicyStatus::InProgress || next == LegacyUsagePolicyStatus::Cancelled;
            }
            LegacyUsagePolicyStatus::Scheduled => {
                match next {
                    LegacyUsagePolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsagePolicyStatus::InProgress => {
                return next == LegacyUsagePolicyStatus::Completed;
            }
            LegacyUsagePolicyStatus::Completed => {
                match next {
                    LegacyUsagePolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyUsagePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy usage policy {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyUsagePolicy {
        LegacyUsagePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            ends_at: self.ends_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy usage policy in status planned.
pub fn new_legacy_usage_policy(code: &str, name: &str) -> LegacyUsagePolicy {
    LegacyUsagePolicy {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        ends_at: 0,
        quantity: 0,
        status: LegacyUsagePolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_usage_policy("code-225", "name-125");
        assert_eq!(item.status, LegacyUsagePolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_legacy_usage_policy("code-225", "name-125");
        if item.move_to(LegacyUsagePolicyStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyUsagePolicyStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_usage_policy("code-225", "name-125");
        item.status = LegacyUsagePolicyStatus::Cancelled;
        assert!(
            item.move_to(LegacyUsagePolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
