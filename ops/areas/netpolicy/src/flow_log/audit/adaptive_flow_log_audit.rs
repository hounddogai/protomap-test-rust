use super::*;

use acme_kit::text;

/// Where a adaptive flow log audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFlowLogAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveFlowLogAuditStatus {
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

impl std::fmt::Display for AdaptiveFlowLogAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive flow log audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogAudit {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub ceiling: f64,
    pub starts_at: i64,
    pub status: AdaptiveFlowLogAuditStatus,
}

impl AdaptiveFlowLogAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFlowLogAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFlowLogAuditStatus) -> bool {
        match self.status {
            AdaptiveFlowLogAuditStatus::Idle => {
                match next {
                    AdaptiveFlowLogAuditStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogAuditStatus::Warming => {
                match next {
                    AdaptiveFlowLogAuditStatus::Draining | AdaptiveFlowLogAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogAuditStatus::Ready => {
                return next == AdaptiveFlowLogAuditStatus::Draining || next == AdaptiveFlowLogAuditStatus::Stopped;
            }
            AdaptiveFlowLogAuditStatus::Draining => {
                return next == AdaptiveFlowLogAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveFlowLogAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive flow log audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFlowLogAudit {
        AdaptiveFlowLogAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive flow log audit in status idle.
pub fn new_adaptive_flow_log_audit(code: &str, name: &str) -> AdaptiveFlowLogAudit {
    AdaptiveFlowLogAudit {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        ceiling: 0.0,
        starts_at: 0,
        status: AdaptiveFlowLogAuditStatus::Idle,
    }
}
