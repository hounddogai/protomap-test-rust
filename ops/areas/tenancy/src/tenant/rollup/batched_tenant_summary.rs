use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched tenant summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTenantSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedTenantSummaryStatus {
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

impl std::fmt::Display for BatchedTenantSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched tenant summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummary {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub locale: String,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub segment: String,
    pub country: String,
    pub cipher: String,
    pub score: f64,
    pub status: BatchedTenantSummaryStatus,
}

impl BatchedTenantSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTenantSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTenantSummaryStatus) -> bool {
        match self.status {
            BatchedTenantSummaryStatus::Idle => {
                return next == BatchedTenantSummaryStatus::Warming;
            }
            BatchedTenantSummaryStatus::Warming => {
                return next == BatchedTenantSummaryStatus::Ready || next == BatchedTenantSummaryStatus::Stopped;
            }
            BatchedTenantSummaryStatus::Ready => {
                return next == BatchedTenantSummaryStatus::Draining || next == BatchedTenantSummaryStatus::Stopped;
            }
            BatchedTenantSummaryStatus::Draining => {
                return next == BatchedTenantSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTenantSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched tenant summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched tenant summary", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> BatchedTenantSummary {
        BatchedTenantSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            locale: self.locale.clone(),
            attributes: self.attributes.clone(),
            approved: self.approved,
            segment: self.segment.clone(),
            country: self.country.clone(),
            cipher: self.cipher.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched tenant summary in status idle.
pub fn new_batched_tenant_summary(code: &str, label: &str) -> BatchedTenantSummary {
    BatchedTenantSummary {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        locale: "".to_string(),
        attributes: BTreeMap::new(),
        approved: false,
        segment: "".to_string(),
        country: "".to_string(),
        cipher: "".to_string(),
        score: 0.0,
        status: BatchedTenantSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_tenant_summary("code-77", "label-495");
        assert_eq!(item.status, BatchedTenantSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_tenant_summary("code-77", "label-495");
        if item.move_to(BatchedTenantSummaryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTenantSummaryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_tenant_summary("code-77", "label-495");
        item.status = BatchedTenantSummaryStatus::Stopped;
        assert!(
            item.move_to(BatchedTenantSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
