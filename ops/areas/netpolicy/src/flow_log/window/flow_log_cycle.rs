use super::*;

use acme_kit::text;

/// Where a flow log cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FlowLogCycleStatus {
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

impl std::fmt::Display for FlowLogCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycle {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub minimum: f64,
    pub external_ref: String,
    pub cipher: String,
    pub rate: f64,
    pub retries: i64,
    pub owner_id: String,
    pub summary: String,
    pub status: FlowLogCycleStatus,
}

impl FlowLogCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogCycleStatus) -> bool {
        match self.status {
            FlowLogCycleStatus::Idle => {
                match next {
                    FlowLogCycleStatus::Draining | FlowLogCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogCycleStatus::Warming => {
                return next == FlowLogCycleStatus::Draining;
            }
            FlowLogCycleStatus::Ready => {
                match next {
                    FlowLogCycleStatus::Draining | FlowLogCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogCycleStatus::Draining => {
                match next {
                    FlowLogCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FlowLogCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.minimum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogCycle {
        FlowLogCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            rate: self.rate,
            retries: self.retries,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a flow log cycle in status idle.
pub fn new_flow_log_cycle(code: &str, label: &str) -> FlowLogCycle {
    FlowLogCycle {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        minimum: 0.0,
        external_ref: "".to_string(),
        cipher: "".to_string(),
        rate: 0.0,
        retries: 0,
        owner_id: "".to_string(),
        summary: "".to_string(),
        status: FlowLogCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_flow_log_cycle("code-607", "label-695");
        assert_eq!(item.status, FlowLogCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_flow_log_cycle("code-607", "label-695");
        if item.move_to(FlowLogCycleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogCycleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_flow_log_cycle("code-607", "label-695");
        item.status = FlowLogCycleStatus::Stopped;
        assert!(item.move_to(FlowLogCycleStatus::Idle).is_err(), "expected move to fail");
    }
}
