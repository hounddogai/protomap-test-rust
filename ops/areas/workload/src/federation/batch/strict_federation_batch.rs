use super::*;

use acme_kit::text;

/// Where a strict federation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFederationBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictFederationBatchStatus {
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

impl std::fmt::Display for StrictFederationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict federation batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatch {
    pub code: String,
    pub label: String,
    pub category: String,
    pub unit: String,
    pub tags: Vec<String>,
    pub tenant_id: String,
    pub port: i64,
    pub score: f64,
    pub rate: f64,
    pub status: StrictFederationBatchStatus,
}

impl StrictFederationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFederationBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFederationBatchStatus) -> bool {
        match self.status {
            StrictFederationBatchStatus::Idle => {
                return next == StrictFederationBatchStatus::Draining || next == StrictFederationBatchStatus::Stopped;
            }
            StrictFederationBatchStatus::Warming => {
                match next {
                    StrictFederationBatchStatus::Draining | StrictFederationBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationBatchStatus::Ready => {
                match next {
                    StrictFederationBatchStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationBatchStatus::Draining => {
                return next == StrictFederationBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFederationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict federation batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFederationBatch {
        StrictFederationBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            tags: self.tags.clone(),
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            score: self.score,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict federation batch in status idle.
pub fn new_strict_federation_batch(code: &str, label: &str) -> StrictFederationBatch {
    StrictFederationBatch {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        tags: Vec::new(),
        tenant_id: "".to_string(),
        port: 0,
        score: 0.0,
        rate: 0.0,
        status: StrictFederationBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_federation_batch("code-44", "label-410");
        assert_eq!(item.status, StrictFederationBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_strict_federation_batch("code-44", "label-410");
        if item.move_to(StrictFederationBatchStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFederationBatchStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_federation_batch("code-44", "label-410");
        item.status = StrictFederationBatchStatus::Stopped;
        assert!(
            item.move_to(StrictFederationBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
