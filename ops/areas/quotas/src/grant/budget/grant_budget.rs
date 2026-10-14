use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a grant budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GrantBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for GrantBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantBudget {
    pub code: String,
    pub name: String,
    pub region: String,
    pub host: String,
    pub archived: bool,
    pub batch_size: i64,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub status: GrantBudgetStatus,
}

impl GrantBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantBudgetStatus) -> bool {
        match self.status {
            GrantBudgetStatus::Idle => {
                match next {
                    GrantBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBudgetStatus::Warming => {
                match next {
                    GrantBudgetStatus::Ready | GrantBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBudgetStatus::Ready => {
                return next == GrantBudgetStatus::Draining || next == GrantBudgetStatus::Stopped;
            }
            GrantBudgetStatus::Draining => {
                return next == GrantBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> GrantBudget {
        GrantBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            host: self.host.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant budget in status idle.
pub fn new_grant_budget(code: &str, name: &str) -> GrantBudget {
    GrantBudget {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        host: "".to_string(),
        archived: false,
        batch_size: 0,
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        status: GrantBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_grant_budget("code-610", "name-359");
        assert_eq!(item.status, GrantBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_grant_budget("code-610", "name-359");
        if item.move_to(GrantBudgetStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantBudgetStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_grant_budget("code-610", "name-359");
        item.status = GrantBudgetStatus::Stopped;
        assert!(item.move_to(GrantBudgetStatus::Idle).is_err(), "expected move to fail");
    }
}
