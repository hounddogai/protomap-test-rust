use super::*;

use acme_kit::text;

/// Where a adaptive flow log template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFlowLogTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveFlowLogTemplateStatus {
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

impl std::fmt::Display for AdaptiveFlowLogTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive flow log template as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogTemplate {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub version: i64,
    pub expires_at: i64,
    pub floor_value: f64,
    pub ttl_seconds: i64,
    pub capacity: i64,
    pub status: AdaptiveFlowLogTemplateStatus,
}

impl AdaptiveFlowLogTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFlowLogTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFlowLogTemplateStatus) -> bool {
        match self.status {
            AdaptiveFlowLogTemplateStatus::Open => {
                match next {
                    AdaptiveFlowLogTemplateStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogTemplateStatus::InReview => {
                return next == AdaptiveFlowLogTemplateStatus::Resolved;
            }
            AdaptiveFlowLogTemplateStatus::Resolved => {
                match next {
                    AdaptiveFlowLogTemplateStatus::Reopened | AdaptiveFlowLogTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogTemplateStatus::Reopened => {
                match next {
                    AdaptiveFlowLogTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveFlowLogTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive flow log template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.floor_value * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFlowLogTemplate {
        AdaptiveFlowLogTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            version: self.version,
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            ttl_seconds: self.ttl_seconds,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive flow log template in status open.
pub fn new_adaptive_flow_log_template(code: &str, label: &str) -> AdaptiveFlowLogTemplate {
    AdaptiveFlowLogTemplate {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        version: 0,
        expires_at: 0,
        floor_value: 0.0,
        ttl_seconds: 0,
        capacity: 0,
        status: AdaptiveFlowLogTemplateStatus::Open,
    }
}
