use super::*;

use acme_kit::text;

/// Where a batched runbook bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedRunbookBundleStatus {
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

impl std::fmt::Display for BatchedRunbookBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched runbook bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundle {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub source: String,
    pub active: bool,
    pub budget: f64,
    pub cluster: String,
    pub visible: bool,
    pub maximum: f64,
    pub status: BatchedRunbookBundleStatus,
}

impl BatchedRunbookBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookBundleStatus) -> bool {
        match self.status {
            BatchedRunbookBundleStatus::Idle => {
                return next == BatchedRunbookBundleStatus::Stopped;
            }
            BatchedRunbookBundleStatus::Warming => {
                match next {
                    BatchedRunbookBundleStatus::Draining | BatchedRunbookBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookBundleStatus::Ready => {
                return next == BatchedRunbookBundleStatus::Draining;
            }
            BatchedRunbookBundleStatus::Draining => {
                return next == BatchedRunbookBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRunbookBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched runbook bundle", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.maximum + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRunbookBundle {
        BatchedRunbookBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            source: self.source.clone(),
            active: self.active,
            budget: self.budget,
            cluster: self.cluster.clone(),
            visible: self.visible,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched runbook bundle in status idle.
pub fn new_batched_runbook_bundle(code: &str, name: &str) -> BatchedRunbookBundle {
    BatchedRunbookBundle {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        source: "".to_string(),
        active: false,
        budget: 0.0,
        cluster: "".to_string(),
        visible: false,
        maximum: 0.0,
        status: BatchedRunbookBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_runbook_bundle("code-391", "name-774");
        assert_eq!(item.status, BatchedRunbookBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_batched_runbook_bundle("code-391", "name-774");
        if item.move_to(BatchedRunbookBundleStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRunbookBundleStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_runbook_bundle("code-391", "name-774");
        item.status = BatchedRunbookBundleStatus::Stopped;
        assert!(
            item.move_to(BatchedRunbookBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
