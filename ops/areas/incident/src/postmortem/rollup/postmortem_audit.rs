use super::*;

use acme_kit::text;

/// Where a postmortem audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemAuditStatus {
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

impl std::fmt::Display for PostmortemAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAudit {
    pub code: String,
    pub label: String,
    pub description: String,
    pub score: f64,
    pub region: String,
    pub status: PostmortemAuditStatus,
}

impl PostmortemAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemAuditStatus) -> bool {
        match self.status {
            PostmortemAuditStatus::Idle => {
                match next {
                    PostmortemAuditStatus::Draining | PostmortemAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemAuditStatus::Warming => {
                match next {
                    PostmortemAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemAuditStatus::Ready => {
                return next == PostmortemAuditStatus::Stopped;
            }
            PostmortemAuditStatus::Draining => {
                match next {
                    PostmortemAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PostmortemAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PostmortemAudit {
        PostmortemAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            score: self.score,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a postmortem audit in status idle.
pub fn new_postmortem_audit(code: &str, label: &str) -> PostmortemAudit {
    PostmortemAudit {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        score: 0.0,
        region: "".to_string(),
        status: PostmortemAuditStatus::Idle,
    }
}
