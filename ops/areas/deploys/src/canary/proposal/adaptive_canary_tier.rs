use super::*;

use acme_kit::text;

/// Where a adaptive canary tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCanaryTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveCanaryTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveCanaryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive canary tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryTier {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub revision: i64,
    pub key_id: String,
    pub status: AdaptiveCanaryTierStatus,
}

impl AdaptiveCanaryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCanaryTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCanaryTierStatus) -> bool {
        match self.status {
            AdaptiveCanaryTierStatus::Proposed => {
                match next {
                    AdaptiveCanaryTierStatus::Approved | AdaptiveCanaryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryTierStatus::Approved => {
                match next {
                    AdaptiveCanaryTierStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryTierStatus::Rejected => {
                match next {
                    AdaptiveCanaryTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveCanaryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive canary tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.attempts as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveCanaryTier {
        AdaptiveCanaryTier {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            revision: self.revision,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive canary tier in status proposed.
pub fn new_adaptive_canary_tier(code: &str, label: &str) -> AdaptiveCanaryTier {
    AdaptiveCanaryTier {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        revision: 0,
        key_id: "".to_string(),
        status: AdaptiveCanaryTierStatus::Proposed,
    }
}
