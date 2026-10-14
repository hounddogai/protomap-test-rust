use super::*;

use acme_kit::text;

/// Where a manual failover cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFailoverCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualFailoverCycleStatus {
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

impl std::fmt::Display for ManualFailoverCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual failover cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverCycle {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub ceiling: f64,
    pub country: String,
    pub status: ManualFailoverCycleStatus,
}

impl ManualFailoverCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFailoverCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFailoverCycleStatus) -> bool {
        match self.status {
            ManualFailoverCycleStatus::Open => {
                match next {
                    ManualFailoverCycleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFailoverCycleStatus::InReview => {
                return next == ManualFailoverCycleStatus::Reopened;
            }
            ManualFailoverCycleStatus::Resolved => {
                match next {
                    ManualFailoverCycleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFailoverCycleStatus::Reopened => {
                return next == ManualFailoverCycleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualFailoverCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual failover cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> ManualFailoverCycle {
        ManualFailoverCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            ceiling: self.ceiling,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual failover cycle in status open.
pub fn new_manual_failover_cycle(code: &str, title: &str) -> ManualFailoverCycle {
    ManualFailoverCycle {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        ceiling: 0.0,
        country: "".to_string(),
        status: ManualFailoverCycleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_failover_cycle("code-506", "title-322");
        assert_eq!(item.status, ManualFailoverCycleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_manual_failover_cycle("code-506", "title-322");
        if item.move_to(ManualFailoverCycleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFailoverCycleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_failover_cycle("code-506", "title-322");
        item.status = ManualFailoverCycleStatus::Archived;
        assert!(
            item.move_to(ManualFailoverCycleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
