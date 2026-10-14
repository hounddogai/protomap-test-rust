use super::*;

use acme_kit::text;

/// Where a flow log draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FlowLogDraftStatus {
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

impl std::fmt::Display for FlowLogDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraft {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub ends_at: i64,
    pub active: bool,
    pub region: String,
    pub channel: String,
    pub archived: bool,
    pub window_days: i64,
    pub status: FlowLogDraftStatus,
}

impl FlowLogDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogDraftStatus) -> bool {
        match self.status {
            FlowLogDraftStatus::Idle => {
                return next == FlowLogDraftStatus::Draining;
            }
            FlowLogDraftStatus::Warming => {
                match next {
                    FlowLogDraftStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogDraftStatus::Ready => {
                match next {
                    FlowLogDraftStatus::Draining | FlowLogDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogDraftStatus::Draining => {
                match next {
                    FlowLogDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FlowLogDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log draft", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ends_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogDraft {
        FlowLogDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            ends_at: self.ends_at,
            active: self.active,
            region: self.region.clone(),
            channel: self.channel.clone(),
            archived: self.archived,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a flow log draft in status idle.
pub fn new_flow_log_draft(code: &str, label: &str) -> FlowLogDraft {
    FlowLogDraft {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        ends_at: 0,
        active: false,
        region: "".to_string(),
        channel: "".to_string(),
        archived: false,
        window_days: 0,
        status: FlowLogDraftStatus::Idle,
    }
}
