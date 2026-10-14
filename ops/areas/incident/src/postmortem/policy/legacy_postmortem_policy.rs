use super::*;

use acme_kit::text;

/// Where a legacy postmortem policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPostmortemPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyPostmortemPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyPostmortemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy postmortem policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPolicy {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub created_at: i64,
    pub priority: i64,
    pub source: String,
    pub budget: f64,
    pub reviewer: String,
    pub quantity: i64,
    pub threshold: f64,
    pub status: LegacyPostmortemPolicyStatus,
}

impl LegacyPostmortemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPostmortemPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPostmortemPolicyStatus) -> bool {
        match self.status {
            LegacyPostmortemPolicyStatus::Proposed => {
                match next {
                    LegacyPostmortemPolicyStatus::Approved | LegacyPostmortemPolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPostmortemPolicyStatus::Approved => {
                match next {
                    LegacyPostmortemPolicyStatus::Rejected | LegacyPostmortemPolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPostmortemPolicyStatus::Rejected => {
                return next == LegacyPostmortemPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPostmortemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy postmortem policy {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "legacy postmortem policy", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.budget * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPostmortemPolicy {
        LegacyPostmortemPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            created_at: self.created_at,
            priority: self.priority,
            source: self.source.clone(),
            budget: self.budget,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy postmortem policy in status proposed.
pub fn new_legacy_postmortem_policy(code: &str, title: &str) -> LegacyPostmortemPolicy {
    LegacyPostmortemPolicy {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        created_at: 0,
        priority: 0,
        source: "".to_string(),
        budget: 0.0,
        reviewer: "".to_string(),
        quantity: 0,
        threshold: 0.0,
        status: LegacyPostmortemPolicyStatus::Proposed,
    }
}
