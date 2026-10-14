use super::*;

use acme_kit::text;

/// Where a global plan window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalPlanWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalPlanWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global plan window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanWindow {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub limit: i64,
    pub margin: f64,
    pub principal: String,
    pub cluster: String,
    pub status: GlobalPlanWindowStatus,
}

impl GlobalPlanWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanWindowStatus) -> bool {
        match self.status {
            GlobalPlanWindowStatus::Requested => {
                match next {
                    GlobalPlanWindowStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanWindowStatus::Granted => {
                match next {
                    GlobalPlanWindowStatus::Revoked | GlobalPlanWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanWindowStatus::Revoked => {
                return next == GlobalPlanWindowStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPlanWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global plan window", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanWindow {
        GlobalPlanWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            limit: self.limit,
            margin: self.margin,
            principal: self.principal.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global plan window in status requested.
pub fn new_global_plan_window(code: &str, title: &str) -> GlobalPlanWindow {
    GlobalPlanWindow {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        limit: 0,
        margin: 0.0,
        principal: "".to_string(),
        cluster: "".to_string(),
        status: GlobalPlanWindowStatus::Requested,
    }
}
