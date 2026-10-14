use super::*;

use acme_kit::text;

/// Where a sink audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkAuditStatus {
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

impl std::fmt::Display for SinkAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink audit as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAudit {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub floor_value: f64,
    pub budget: f64,
    pub status: SinkAuditStatus,
}

impl SinkAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkAuditStatus) -> bool {
        match self.status {
            SinkAuditStatus::Idle => {
                return next == SinkAuditStatus::Ready || next == SinkAuditStatus::Draining;
            }
            SinkAuditStatus::Warming => {
                match next {
                    SinkAuditStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAuditStatus::Ready => {
                match next {
                    SinkAuditStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAuditStatus::Draining => {
                return next == SinkAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink audit", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.floor_value * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkAudit {
        SinkAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink audit in status idle.
pub fn new_sink_audit(code: &str, label: &str) -> SinkAudit {
    SinkAudit {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        floor_value: 0.0,
        budget: 0.0,
        status: SinkAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_sink_audit("code-562", "label-444");
        assert_eq!(item.status, SinkAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_sink_audit("code-562", "label-444");
        if item.move_to(SinkAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_sink_audit("code-562", "label-444");
        item.status = SinkAuditStatus::Stopped;
        assert!(item.move_to(SinkAuditStatus::Idle).is_err(), "expected move to fail");
    }
}
