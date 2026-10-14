use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict postmortem plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPostmortemPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPostmortemPlanStatus {
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

impl std::fmt::Display for StrictPostmortemPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict postmortem plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlan {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub active: bool,
    pub grace_days: i64,
    pub status: StrictPostmortemPlanStatus,
}

impl StrictPostmortemPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPostmortemPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPostmortemPlanStatus) -> bool {
        match self.status {
            StrictPostmortemPlanStatus::Open => {
                return next == StrictPostmortemPlanStatus::InReview || next == StrictPostmortemPlanStatus::Resolved;
            }
            StrictPostmortemPlanStatus::InReview => {
                match next {
                    StrictPostmortemPlanStatus::Resolved | StrictPostmortemPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPostmortemPlanStatus::Resolved => {
                match next {
                    StrictPostmortemPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPostmortemPlanStatus::Reopened => {
                return next == StrictPostmortemPlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPostmortemPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict postmortem plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPostmortemPlan {
        StrictPostmortemPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            active: self.active,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict postmortem plan in status open.
pub fn new_strict_postmortem_plan(code: &str, label: &str) -> StrictPostmortemPlan {
    StrictPostmortemPlan {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        active: false,
        grace_days: 0,
        status: StrictPostmortemPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_postmortem_plan("code-284", "label-739");
        assert_eq!(item.status, StrictPostmortemPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_strict_postmortem_plan("code-284", "label-739");
        if item.move_to(StrictPostmortemPlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPostmortemPlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_postmortem_plan("code-284", "label-739");
        item.status = StrictPostmortemPlanStatus::Archived;
        assert!(
            item.move_to(StrictPostmortemPlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
