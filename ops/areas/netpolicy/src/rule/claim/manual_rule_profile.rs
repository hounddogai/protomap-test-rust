use super::*;

use acme_kit::text;

/// Where a manual rule profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRuleProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualRuleProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualRuleProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rule profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfile {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub score: f64,
    pub margin: f64,
    pub segment: String,
    pub status: ManualRuleProfileStatus,
}

impl ManualRuleProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRuleProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRuleProfileStatus) -> bool {
        match self.status {
            ManualRuleProfileStatus::Proposed => {
                return next == ManualRuleProfileStatus::Approved || next == ManualRuleProfileStatus::Withdrawn;
            }
            ManualRuleProfileStatus::Approved => {
                match next {
                    ManualRuleProfileStatus::Rejected | ManualRuleProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRuleProfileStatus::Rejected => {
                match next {
                    ManualRuleProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualRuleProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rule profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual rule profile", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.score + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRuleProfile {
        ManualRuleProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            score: self.score,
            margin: self.margin,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual rule profile in status proposed.
pub fn new_manual_rule_profile(code: &str, title: &str) -> ManualRuleProfile {
    ManualRuleProfile {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        score: 0.0,
        margin: 0.0,
        segment: "".to_string(),
        status: ManualRuleProfileStatus::Proposed,
    }
}
