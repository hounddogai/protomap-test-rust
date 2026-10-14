use super::*;

use acme_kit::text;

/// Where a seasonal nonce allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalNonceAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalNonceAllocationStatus {
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

impl std::fmt::Display for SeasonalNonceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal nonce allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocation {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub ttl_seconds: i64,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub tags: Vec<String>,
    pub burst: i64,
    pub namespace: String,
    pub cluster: String,
    pub status: SeasonalNonceAllocationStatus,
}

impl SeasonalNonceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalNonceAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalNonceAllocationStatus) -> bool {
        match self.status {
            SeasonalNonceAllocationStatus::Idle => {
                return next == SeasonalNonceAllocationStatus::Draining
                    || next == SeasonalNonceAllocationStatus::Stopped;
            }
            SeasonalNonceAllocationStatus::Warming => {
                match next {
                    SeasonalNonceAllocationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalNonceAllocationStatus::Ready => {
                return next == SeasonalNonceAllocationStatus::Stopped;
            }
            SeasonalNonceAllocationStatus::Draining => {
                return next == SeasonalNonceAllocationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalNonceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal nonce allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal nonce allocation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalNonceAllocation {
        SeasonalNonceAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            tags: self.tags.clone(),
            burst: self.burst,
            namespace: self.namespace.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal nonce allocation in status idle.
pub fn new_seasonal_nonce_allocation(code: &str, label: &str) -> SeasonalNonceAllocation {
    SeasonalNonceAllocation {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        ttl_seconds: 0,
        duration_minutes: 0,
        external_ref: "".to_string(),
        tags: Vec::new(),
        burst: 0,
        namespace: "".to_string(),
        cluster: "".to_string(),
        status: SeasonalNonceAllocationStatus::Idle,
    }
}
