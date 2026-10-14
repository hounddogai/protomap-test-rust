use super::*;

use acme_kit::text;

/// Where a legacy grant baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyGrantBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyGrantBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyGrantBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy grant baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaseline {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub port: i64,
    pub maximum: f64,
    pub amount: f64,
    pub grace_days: i64,
    pub status: LegacyGrantBaselineStatus,
}

impl LegacyGrantBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyGrantBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyGrantBaselineStatus) -> bool {
        match self.status {
            LegacyGrantBaselineStatus::Proposed => {
                match next {
                    LegacyGrantBaselineStatus::Rejected | LegacyGrantBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantBaselineStatus::Approved => {
                return next == LegacyGrantBaselineStatus::Rejected;
            }
            LegacyGrantBaselineStatus::Rejected => {
                return next == LegacyGrantBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyGrantBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy grant baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyGrantBaseline {
        LegacyGrantBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            port: self.port,
            maximum: self.maximum,
            amount: self.amount,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy grant baseline in status proposed.
pub fn new_legacy_grant_baseline(code: &str, name: &str) -> LegacyGrantBaseline {
    LegacyGrantBaseline {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        port: 0,
        maximum: 0.0,
        amount: 0.0,
        grace_days: 0,
        status: LegacyGrantBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_grant_baseline("code-496", "name-703");
        assert_eq!(item.status, LegacyGrantBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_legacy_grant_baseline("code-496", "name-703");
        if item.move_to(LegacyGrantBaselineStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyGrantBaselineStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_grant_baseline("code-496", "name-703");
        item.status = LegacyGrantBaselineStatus::Withdrawn;
        assert!(
            item.move_to(LegacyGrantBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
