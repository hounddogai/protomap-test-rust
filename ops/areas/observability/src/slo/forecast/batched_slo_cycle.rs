use super::*;

use acme_kit::text;

/// Where a batched slo cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSloCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedSloCycleStatus {
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

impl std::fmt::Display for BatchedSloCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched slo cycle as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloCycle {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub summary: String,
    pub label: String,
    pub source: String,
    pub host: String,
    pub ceiling: f64,
    pub audience: String,
    pub status: BatchedSloCycleStatus,
}

impl BatchedSloCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSloCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSloCycleStatus) -> bool {
        match self.status {
            BatchedSloCycleStatus::Idle => {
                return next == BatchedSloCycleStatus::Warming || next == BatchedSloCycleStatus::Draining;
            }
            BatchedSloCycleStatus::Warming => {
                match next {
                    BatchedSloCycleStatus::Ready | BatchedSloCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloCycleStatus::Ready => {
                match next {
                    BatchedSloCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloCycleStatus::Draining => {
                return next == BatchedSloCycleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSloCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched slo cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> BatchedSloCycle {
        BatchedSloCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            host: self.host.clone(),
            ceiling: self.ceiling,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a batched slo cycle in status idle.
pub fn new_batched_slo_cycle(code: &str, name: &str) -> BatchedSloCycle {
    BatchedSloCycle {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        label: "".to_string(),
        source: "".to_string(),
        host: "".to_string(),
        ceiling: 0.0,
        audience: "".to_string(),
        status: BatchedSloCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_slo_cycle("code-891", "name-933");
        assert_eq!(item.status, BatchedSloCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_slo_cycle("code-891", "name-933");
        if item.move_to(BatchedSloCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSloCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_slo_cycle("code-891", "name-933");
        item.status = BatchedSloCycleStatus::Stopped;
        assert!(
            item.move_to(BatchedSloCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
