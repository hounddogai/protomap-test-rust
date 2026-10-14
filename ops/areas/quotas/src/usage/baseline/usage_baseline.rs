use super::*;

use acme_kit::text;

/// Where a usage baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaseline {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub currency: String,
    pub audience: String,
    pub score: f64,
    pub status: UsageBaselineStatus,
}

impl UsageBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageBaselineStatus) -> bool {
        match self.status {
            UsageBaselineStatus::Proposed => {
                match next {
                    UsageBaselineStatus::Rejected | UsageBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBaselineStatus::Approved => {
                return next == UsageBaselineStatus::Withdrawn;
            }
            UsageBaselineStatus::Rejected => {
                match next {
                    UsageBaselineStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: UsageBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> UsageBaseline {
        UsageBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            currency: self.currency.clone(),
            audience: self.audience.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage baseline in status proposed.
pub fn new_usage_baseline(code: &str, name: &str) -> UsageBaseline {
    UsageBaseline {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        currency: "".to_string(),
        audience: "".to_string(),
        score: 0.0,
        status: UsageBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_baseline("code-537", "name-156");
        assert_eq!(item.status, UsageBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_usage_baseline("code-537", "name-156");
        if item.move_to(UsageBaselineStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageBaselineStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_baseline("code-537", "name-156");
        item.status = UsageBaselineStatus::Withdrawn;
        assert!(
            item.move_to(UsageBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
