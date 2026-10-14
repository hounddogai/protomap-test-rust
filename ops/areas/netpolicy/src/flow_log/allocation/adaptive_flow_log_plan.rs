use super::*;

use acme_kit::text;

/// Where a adaptive flow log plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFlowLogPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveFlowLogPlanStatus {
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

impl std::fmt::Display for AdaptiveFlowLogPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive flow log plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlan {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub status: AdaptiveFlowLogPlanStatus,
}

impl AdaptiveFlowLogPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFlowLogPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFlowLogPlanStatus) -> bool {
        match self.status {
            AdaptiveFlowLogPlanStatus::New => {
                match next {
                    AdaptiveFlowLogPlanStatus::Queued | AdaptiveFlowLogPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogPlanStatus::Queued => {
                match next {
                    AdaptiveFlowLogPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogPlanStatus::Running => {
                match next {
                    AdaptiveFlowLogPlanStatus::Done | AdaptiveFlowLogPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogPlanStatus::Done => {
                return next == AdaptiveFlowLogPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveFlowLogPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive flow log plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFlowLogPlan {
        AdaptiveFlowLogPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            burst: self.burst,
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive flow log plan in status new.
pub fn new_adaptive_flow_log_plan(code: &str, name: &str) -> AdaptiveFlowLogPlan {
    AdaptiveFlowLogPlan {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        burst: 0,
        quota_units: 0,
        tags: Vec::new(),
        status: AdaptiveFlowLogPlanStatus::New,
    }
}
