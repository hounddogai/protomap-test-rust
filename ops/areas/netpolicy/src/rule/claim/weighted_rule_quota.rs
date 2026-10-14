use super::*;

use acme_kit::text;

/// Where a weighted rule quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedRuleQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedRuleQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted rule quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuota {
    pub code: String,
    pub label: String,
    pub title: String,
    pub locale: String,
    pub budget: f64,
    pub status: WeightedRuleQuotaStatus,
}

impl WeightedRuleQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleQuotaStatus) -> bool {
        match self.status {
            WeightedRuleQuotaStatus::Idle => {
                match next {
                    WeightedRuleQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleQuotaStatus::Warming => {
                return next == WeightedRuleQuotaStatus::Ready;
            }
            WeightedRuleQuotaStatus::Ready => {
                match next {
                    WeightedRuleQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleQuotaStatus::Draining => {
                return next == WeightedRuleQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRuleQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> WeightedRuleQuota {
        WeightedRuleQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted rule quota in status idle.
pub fn new_weighted_rule_quota(code: &str, label: &str) -> WeightedRuleQuota {
    WeightedRuleQuota {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        locale: "".to_string(),
        budget: 0.0,
        status: WeightedRuleQuotaStatus::Idle,
    }
}
