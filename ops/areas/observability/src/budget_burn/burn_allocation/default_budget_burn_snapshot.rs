use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default budget burn snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultBudgetBurnSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultBudgetBurnSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultBudgetBurnSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default budget burn snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshot {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub attempts: i64,
    pub attributes: BTreeMap<String, String>,
    pub label: String,
    pub ratio: f64,
    pub status: DefaultBudgetBurnSnapshotStatus,
}

impl DefaultBudgetBurnSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultBudgetBurnSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultBudgetBurnSnapshotStatus) -> bool {
        match self.status {
            DefaultBudgetBurnSnapshotStatus::Requested => {
                match next {
                    DefaultBudgetBurnSnapshotStatus::Granted | DefaultBudgetBurnSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnSnapshotStatus::Granted => {
                match next {
                    DefaultBudgetBurnSnapshotStatus::Revoked | DefaultBudgetBurnSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnSnapshotStatus::Revoked => {
                match next {
                    DefaultBudgetBurnSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultBudgetBurnSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default budget burn snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default budget burn snapshot", self.code)
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

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultBudgetBurnSnapshot {
        DefaultBudgetBurnSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            attempts: self.attempts,
            attributes: self.attributes.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default budget burn snapshot in status requested.
pub fn new_default_budget_burn_snapshot(code: &str, title: &str) -> DefaultBudgetBurnSnapshot {
    DefaultBudgetBurnSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        attempts: 0,
        attributes: BTreeMap::new(),
        label: "".to_string(),
        ratio: 0.0,
        status: DefaultBudgetBurnSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_budget_burn_snapshot("code-486", "title-181");
        assert_eq!(item.status, DefaultBudgetBurnSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_budget_burn_snapshot("code-486", "title-181");
        if item.move_to(DefaultBudgetBurnSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultBudgetBurnSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_budget_burn_snapshot("code-486", "title-181");
        item.status = DefaultBudgetBurnSnapshotStatus::Expired;
        assert!(
            item.move_to(DefaultBudgetBurnSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
