use super::*;

use acme_kit::text;

/// Where a manual delegation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDelegationOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualDelegationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualDelegationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual delegation override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverride {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub amount: f64,
    pub capacity: i64,
    pub label: String,
    pub weight: f64,
    pub status: ManualDelegationOverrideStatus,
}

impl ManualDelegationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDelegationOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDelegationOverrideStatus) -> bool {
        match self.status {
            ManualDelegationOverrideStatus::New => {
                match next {
                    ManualDelegationOverrideStatus::Running | ManualDelegationOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDelegationOverrideStatus::Queued => {
                match next {
                    ManualDelegationOverrideStatus::Running | ManualDelegationOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDelegationOverrideStatus::Running => {
                match next {
                    ManualDelegationOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDelegationOverrideStatus::Done => {
                return next == ManualDelegationOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDelegationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual delegation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual delegation override", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.retries as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDelegationOverride {
        ManualDelegationOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            amount: self.amount,
            capacity: self.capacity,
            label: self.label.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual delegation override in status new.
pub fn new_manual_delegation_override(code: &str, name: &str) -> ManualDelegationOverride {
    ManualDelegationOverride {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        amount: 0.0,
        capacity: 0,
        label: "".to_string(),
        weight: 0.0,
        status: ManualDelegationOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_delegation_override("code-137", "name-598");
        assert_eq!(item.status, ManualDelegationOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_delegation_override("code-137", "name-598");
        if item.move_to(ManualDelegationOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDelegationOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_delegation_override("code-137", "name-598");
        item.status = ManualDelegationOverrideStatus::Failed;
        assert!(
            item.move_to(ManualDelegationOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
