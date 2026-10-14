use super::*;

use acme_kit::text;

/// Where a rollout rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RolloutRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RolloutRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRule {
    pub code: String,
    pub name: String,
    pub title: String,
    pub attempts: i64,
    pub source: String,
    pub summary: String,
    pub approved: bool,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub discount: f64,
    pub status: RolloutRuleStatus,
}

impl RolloutRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutRuleStatus) -> bool {
        match self.status {
            RolloutRuleStatus::Proposed => {
                return next == RolloutRuleStatus::Rejected;
            }
            RolloutRuleStatus::Approved => {
                return next == RolloutRuleStatus::Rejected || next == RolloutRuleStatus::Withdrawn;
            }
            RolloutRuleStatus::Rejected => {
                return next == RolloutRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.discount - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutRule {
        RolloutRule {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            source: self.source.clone(),
            summary: self.summary.clone(),
            approved: self.approved,
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout rule in status proposed.
pub fn new_rollout_rule(code: &str, name: &str) -> RolloutRule {
    RolloutRule {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        attempts: 0,
        source: "".to_string(),
        summary: "".to_string(),
        approved: false,
        tags: Vec::new(),
        rotation_days: 0,
        discount: 0.0,
        status: RolloutRuleStatus::Proposed,
    }
}
