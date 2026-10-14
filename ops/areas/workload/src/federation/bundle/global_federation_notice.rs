use super::*;

use acme_kit::text;

/// Where a global federation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFederationNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalFederationNoticeStatus {
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

impl std::fmt::Display for GlobalFederationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global federation notice as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationNotice {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub budget: f64,
    pub visible: bool,
    pub segment: String,
    pub notes: String,
    pub discount: f64,
    pub issuer: String,
    pub scope: String,
    pub ceiling: f64,
    pub status: GlobalFederationNoticeStatus,
}

impl GlobalFederationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFederationNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFederationNoticeStatus) -> bool {
        match self.status {
            GlobalFederationNoticeStatus::Idle => {
                return next == GlobalFederationNoticeStatus::Warming || next == GlobalFederationNoticeStatus::Draining;
            }
            GlobalFederationNoticeStatus::Warming => {
                return next == GlobalFederationNoticeStatus::Ready || next == GlobalFederationNoticeStatus::Stopped;
            }
            GlobalFederationNoticeStatus::Ready => {
                match next {
                    GlobalFederationNoticeStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFederationNoticeStatus::Draining => {
                match next {
                    GlobalFederationNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalFederationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global federation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFederationNotice {
        GlobalFederationNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            budget: self.budget,
            visible: self.visible,
            segment: self.segment.clone(),
            notes: self.notes.clone(),
            discount: self.discount,
            issuer: self.issuer.clone(),
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global federation notice in status idle.
pub fn new_global_federation_notice(code: &str, label: &str) -> GlobalFederationNotice {
    GlobalFederationNotice {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        budget: 0.0,
        visible: false,
        segment: "".to_string(),
        notes: "".to_string(),
        discount: 0.0,
        issuer: "".to_string(),
        scope: "".to_string(),
        ceiling: 0.0,
        status: GlobalFederationNoticeStatus::Idle,
    }
}
