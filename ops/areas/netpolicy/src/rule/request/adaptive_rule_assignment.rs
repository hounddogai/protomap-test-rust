use super::*;

use acme_kit::text;

/// Where a adaptive rule assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRuleAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveRuleAssignmentStatus {
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

impl std::fmt::Display for AdaptiveRuleAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive rule assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignment {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub external_ref: String,
    pub title: String,
    pub rotation_days: i64,
    pub discount: f64,
    pub capacity: i64,
    pub pinned: bool,
    pub reviewer: String,
    pub status: AdaptiveRuleAssignmentStatus,
}

impl AdaptiveRuleAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRuleAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRuleAssignmentStatus) -> bool {
        match self.status {
            AdaptiveRuleAssignmentStatus::New => {
                match next {
                    AdaptiveRuleAssignmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRuleAssignmentStatus::Queued => {
                match next {
                    AdaptiveRuleAssignmentStatus::Running | AdaptiveRuleAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRuleAssignmentStatus::Running => {
                return next == AdaptiveRuleAssignmentStatus::Done;
            }
            AdaptiveRuleAssignmentStatus::Done => {
                match next {
                    AdaptiveRuleAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveRuleAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rule assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRuleAssignment {
        AdaptiveRuleAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            external_ref: self.external_ref.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            discount: self.discount,
            capacity: self.capacity,
            pinned: self.pinned,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive rule assignment in status new.
pub fn new_adaptive_rule_assignment(code: &str, label: &str) -> AdaptiveRuleAssignment {
    AdaptiveRuleAssignment {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        external_ref: "".to_string(),
        title: "".to_string(),
        rotation_days: 0,
        discount: 0.0,
        capacity: 0,
        pinned: false,
        reviewer: "".to_string(),
        status: AdaptiveRuleAssignmentStatus::New,
    }
}
