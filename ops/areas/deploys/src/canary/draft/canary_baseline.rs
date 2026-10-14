use super::*;

use acme_kit::text;

/// Where a canary baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBaseline {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub active: bool,
    pub port: i64,
    pub status: CanaryBaselineStatus,
}

impl CanaryBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryBaselineStatus) -> bool {
        match self.status {
            CanaryBaselineStatus::Proposed => {
                match next {
                    CanaryBaselineStatus::Approved | CanaryBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryBaselineStatus::Approved => {
                return next == CanaryBaselineStatus::Withdrawn;
            }
            CanaryBaselineStatus::Rejected => {
                return next == CanaryBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryBaseline {
        CanaryBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            active: self.active,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a canary baseline in status proposed.
pub fn new_canary_baseline(code: &str, title: &str) -> CanaryBaseline {
    CanaryBaseline {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        active: false,
        port: 0,
        status: CanaryBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_baseline("code-990", "title-858");
        assert_eq!(item.status, CanaryBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_canary_baseline("code-990", "title-858");
        if item.move_to(CanaryBaselineStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryBaselineStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_baseline("code-990", "title-858");
        item.status = CanaryBaselineStatus::Withdrawn;
        assert!(
            item.move_to(CanaryBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
