use super::*;

use acme_kit::text;

/// Where a batched usage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedUsageManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedUsageManifestStatus {
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

impl std::fmt::Display for BatchedUsageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched usage manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifest {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub reviewer: String,
    pub quantity: i64,
    pub description: String,
    pub status: BatchedUsageManifestStatus,
}

impl BatchedUsageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedUsageManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedUsageManifestStatus) -> bool {
        match self.status {
            BatchedUsageManifestStatus::Idle => {
                return next == BatchedUsageManifestStatus::Draining;
            }
            BatchedUsageManifestStatus::Warming => {
                return next == BatchedUsageManifestStatus::Draining || next == BatchedUsageManifestStatus::Stopped;
            }
            BatchedUsageManifestStatus::Ready => {
                match next {
                    BatchedUsageManifestStatus::Draining | BatchedUsageManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedUsageManifestStatus::Draining => {
                match next {
                    BatchedUsageManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedUsageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched usage manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedUsageManifest {
        BatchedUsageManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched usage manifest in status idle.
pub fn new_batched_usage_manifest(code: &str, label: &str) -> BatchedUsageManifest {
    BatchedUsageManifest {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        quantity: 0,
        description: "".to_string(),
        status: BatchedUsageManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_usage_manifest("code-311", "label-386");
        assert_eq!(item.status, BatchedUsageManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_batched_usage_manifest("code-311", "label-386");
        if item.move_to(BatchedUsageManifestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedUsageManifestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_usage_manifest("code-311", "label-386");
        item.status = BatchedUsageManifestStatus::Stopped;
        assert!(
            item.move_to(BatchedUsageManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
