use super::*;

use acme_kit::text;

/// Where a fallback rule cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRuleCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackRuleCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackRuleCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback rule cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycle {
    pub code: String,
    pub title: String,
    pub label: String,
    pub namespace: String,
    pub retries: i64,
    pub sequence: i64,
    pub segment: String,
    pub status: FallbackRuleCycleStatus,
}

impl FallbackRuleCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRuleCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRuleCycleStatus) -> bool {
        match self.status {
            FallbackRuleCycleStatus::Proposed => {
                match next {
                    FallbackRuleCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleCycleStatus::Approved => {
                match next {
                    FallbackRuleCycleStatus::Rejected | FallbackRuleCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleCycleStatus::Rejected => {
                match next {
                    FallbackRuleCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackRuleCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rule cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRuleCycle {
        FallbackRuleCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            retries: self.retries,
            sequence: self.sequence,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback rule cycle in status proposed.
pub fn new_fallback_rule_cycle(code: &str, title: &str) -> FallbackRuleCycle {
    FallbackRuleCycle {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        namespace: "".to_string(),
        retries: 0,
        sequence: 0,
        segment: "".to_string(),
        status: FallbackRuleCycleStatus::Proposed,
    }
}
