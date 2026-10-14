use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a limit milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LimitMilestoneStatus {
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

impl std::fmt::Display for LimitMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMilestone {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub rate: f64,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub created_at: i64,
    pub budget: f64,
    pub amount: f64,
    pub category: String,
    pub attempts: i64,
    pub status: LimitMilestoneStatus,
}

impl LimitMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitMilestoneStatus) -> bool {
        match self.status {
            LimitMilestoneStatus::New => {
                return next == LimitMilestoneStatus::Queued;
            }
            LimitMilestoneStatus::Queued => {
                return next == LimitMilestoneStatus::Running;
            }
            LimitMilestoneStatus::Running => {
                return next == LimitMilestoneStatus::Done || next == LimitMilestoneStatus::Failed;
            }
            LimitMilestoneStatus::Done => {
                match next {
                    LimitMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: LimitMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitMilestone {
        LimitMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            rate: self.rate,
            archived: self.archived,
            attributes: self.attributes.clone(),
            created_at: self.created_at,
            budget: self.budget,
            amount: self.amount,
            category: self.category.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a limit milestone in status new.
pub fn new_limit_milestone(code: &str, label: &str) -> LimitMilestone {
    LimitMilestone {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        rate: 0.0,
        archived: false,
        attributes: BTreeMap::new(),
        created_at: 0,
        budget: 0.0,
        amount: 0.0,
        category: "".to_string(),
        attempts: 0,
        status: LimitMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_limit_milestone("code-840", "label-561");
        assert_eq!(item.status, LimitMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_limit_milestone("code-840", "label-561");
        if item.move_to(LimitMilestoneStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitMilestoneStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_limit_milestone("code-840", "label-561");
        item.status = LimitMilestoneStatus::Failed;
        assert!(
            item.move_to(LimitMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
