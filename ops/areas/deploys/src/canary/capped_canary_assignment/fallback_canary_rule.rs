use super::*;

use acme_kit::text;

/// Where a fallback canary rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackCanaryRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackCanaryRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackCanaryRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback canary rule as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRule {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub reviewer: String,
    pub segment: String,
    pub timezone: String,
    pub ceiling: f64,
    pub active: bool,
    pub priority: i64,
    pub status: FallbackCanaryRuleStatus,
}

impl FallbackCanaryRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackCanaryRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackCanaryRuleStatus) -> bool {
        match self.status {
            FallbackCanaryRuleStatus::Proposed => {
                return next == FallbackCanaryRuleStatus::Approved;
            }
            FallbackCanaryRuleStatus::Approved => {
                return next == FallbackCanaryRuleStatus::Withdrawn;
            }
            FallbackCanaryRuleStatus::Rejected => {
                match next {
                    FallbackCanaryRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackCanaryRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback canary rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.priority as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackCanaryRule {
        FallbackCanaryRule {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            active: self.active,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback canary rule in status proposed.
pub fn new_fallback_canary_rule(code: &str, label: &str) -> FallbackCanaryRule {
    FallbackCanaryRule {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        segment: "".to_string(),
        timezone: "".to_string(),
        ceiling: 0.0,
        active: false,
        priority: 0,
        status: FallbackCanaryRuleStatus::Proposed,
    }
}
