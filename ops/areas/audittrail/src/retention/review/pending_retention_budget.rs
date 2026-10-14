use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending retention budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingRetentionBudgetStatus {
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

impl std::fmt::Display for PendingRetentionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending retention budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudget {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub notes: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub revision: i64,
    pub region: String,
    pub tags: Vec<String>,
    pub window_days: i64,
    pub country: String,
    pub status: PendingRetentionBudgetStatus,
}

impl PendingRetentionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionBudgetStatus) -> bool {
        match self.status {
            PendingRetentionBudgetStatus::New => {
                match next {
                    PendingRetentionBudgetStatus::Running | PendingRetentionBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRetentionBudgetStatus::Queued => {
                return next == PendingRetentionBudgetStatus::Failed;
            }
            PendingRetentionBudgetStatus::Running => {
                return next == PendingRetentionBudgetStatus::Done || next == PendingRetentionBudgetStatus::Failed;
            }
            PendingRetentionBudgetStatus::Done => {
                match next {
                    PendingRetentionBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingRetentionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn net_total(&self) -> f64 {
        ((self.revision as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRetentionBudget {
        PendingRetentionBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            notes: self.notes.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            revision: self.revision,
            region: self.region.clone(),
            tags: self.tags.clone(),
            window_days: self.window_days,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending retention budget in status new.
pub fn new_pending_retention_budget(code: &str, label: &str) -> PendingRetentionBudget {
    PendingRetentionBudget {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        notes: "".to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        revision: 0,
        region: "".to_string(),
        tags: Vec::new(),
        window_days: 0,
        country: "".to_string(),
        status: PendingRetentionBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_retention_budget("code-142", "label-420");
        assert_eq!(item.status, PendingRetentionBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_retention_budget("code-142", "label-420");
        if item.move_to(PendingRetentionBudgetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRetentionBudgetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_retention_budget("code-142", "label-420");
        item.status = PendingRetentionBudgetStatus::Failed;
        assert!(
            item.move_to(PendingRetentionBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
