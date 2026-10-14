use super::*;

use acme_kit::text;

/// Where a global dashboard notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDashboardNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalDashboardNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalDashboardNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global dashboard notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNotice {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub ends_at: i64,
    pub ceiling: f64,
    pub status: GlobalDashboardNoticeStatus,
}

impl GlobalDashboardNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDashboardNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDashboardNoticeStatus) -> bool {
        match self.status {
            GlobalDashboardNoticeStatus::Draft => {
                return next == GlobalDashboardNoticeStatus::Suspended;
            }
            GlobalDashboardNoticeStatus::Pending => {
                match next {
                    GlobalDashboardNoticeStatus::Active | GlobalDashboardNoticeStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDashboardNoticeStatus::Active => {
                match next {
                    GlobalDashboardNoticeStatus::Suspended | GlobalDashboardNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDashboardNoticeStatus::Suspended => {
                return next == GlobalDashboardNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalDashboardNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global dashboard notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global dashboard notice", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDashboardNotice {
        GlobalDashboardNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global dashboard notice in status draft.
pub fn new_global_dashboard_notice(code: &str, label: &str) -> GlobalDashboardNotice {
    GlobalDashboardNotice {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        ends_at: 0,
        ceiling: 0.0,
        status: GlobalDashboardNoticeStatus::Draft,
    }
}
