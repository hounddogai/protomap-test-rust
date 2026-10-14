use super::*;

use acme_kit::text;

/// Where a global budget burn notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalBudgetBurnNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalBudgetBurnNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalBudgetBurnNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global budget burn notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNotice {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub visible: bool,
    pub port: i64,
    pub cluster: String,
    pub namespace: String,
    pub fingerprint: String,
    pub notes: String,
    pub margin: f64,
    pub status: GlobalBudgetBurnNoticeStatus,
}

impl GlobalBudgetBurnNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalBudgetBurnNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalBudgetBurnNoticeStatus) -> bool {
        match self.status {
            GlobalBudgetBurnNoticeStatus::Proposed => {
                match next {
                    GlobalBudgetBurnNoticeStatus::Approved | GlobalBudgetBurnNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBudgetBurnNoticeStatus::Approved => {
                return next == GlobalBudgetBurnNoticeStatus::Rejected
                    || next == GlobalBudgetBurnNoticeStatus::Withdrawn;
            }
            GlobalBudgetBurnNoticeStatus::Rejected => {
                return next == GlobalBudgetBurnNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalBudgetBurnNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global budget burn notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.port as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalBudgetBurnNotice {
        GlobalBudgetBurnNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            port: self.port,
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            fingerprint: self.fingerprint.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global budget burn notice in status proposed.
pub fn new_global_budget_burn_notice(code: &str, title: &str) -> GlobalBudgetBurnNotice {
    GlobalBudgetBurnNotice {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        visible: false,
        port: 0,
        cluster: "".to_string(),
        namespace: "".to_string(),
        fingerprint: "".to_string(),
        notes: "".to_string(),
        margin: 0.0,
        status: GlobalBudgetBurnNoticeStatus::Proposed,
    }
}
