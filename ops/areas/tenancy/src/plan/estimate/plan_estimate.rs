use super::*;

use acme_kit::text;

/// Where a plan estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanEstimateStatus {
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

impl std::fmt::Display for PlanEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanEstimate {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub revision: i64,
    pub threshold: f64,
    pub host: String,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub contact_email: String,
    pub priority: i64,
    pub status: PlanEstimateStatus,
}

impl PlanEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanEstimateStatus) -> bool {
        match self.status {
            PlanEstimateStatus::Idle => {
                match next {
                    PlanEstimateStatus::Warming | PlanEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanEstimateStatus::Warming => {
                return next == PlanEstimateStatus::Ready || next == PlanEstimateStatus::Stopped;
            }
            PlanEstimateStatus::Ready => {
                return next == PlanEstimateStatus::Draining || next == PlanEstimateStatus::Stopped;
            }
            PlanEstimateStatus::Draining => {
                match next {
                    PlanEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanEstimate {
        PlanEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            revision: self.revision,
            threshold: self.threshold,
            host: self.host.clone(),
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            created_at: self.created_at,
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan estimate in status idle.
pub fn new_plan_estimate(code: &str, name: &str) -> PlanEstimate {
    PlanEstimate {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        revision: 0,
        threshold: 0.0,
        host: "".to_string(),
        ttl_seconds: 0,
        tags: Vec::new(),
        created_at: 0,
        contact_email: "".to_string(),
        priority: 0,
        status: PlanEstimateStatus::Idle,
    }
}
