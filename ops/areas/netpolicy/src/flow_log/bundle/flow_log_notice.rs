use super::*;

use acme_kit::text;

/// Where a flow log notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogNotice {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub retries: i64,
    pub currency: String,
    pub status: FlowLogNoticeStatus,
}

impl FlowLogNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogNoticeStatus) -> bool {
        match self.status {
            FlowLogNoticeStatus::Proposed => {
                match next {
                    FlowLogNoticeStatus::Approved | FlowLogNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogNoticeStatus::Approved => {
                match next {
                    FlowLogNoticeStatus::Rejected | FlowLogNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogNoticeStatus::Rejected => {
                match next {
                    FlowLogNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log notice", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.created_at as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogNotice {
        FlowLogNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            retries: self.retries,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log notice in status proposed.
pub fn new_flow_log_notice(code: &str, title: &str) -> FlowLogNotice {
    FlowLogNotice {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        retries: 0,
        currency: "".to_string(),
        status: FlowLogNoticeStatus::Proposed,
    }
}
