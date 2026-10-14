use super::*;

use acme_kit::text;

/// Where a delegation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationException {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub ceiling: f64,
    pub segment: String,
    pub namespace: String,
    pub status: DelegationExceptionStatus,
}

impl DelegationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationExceptionStatus) -> bool {
        match self.status {
            DelegationExceptionStatus::Proposed => {
                match next {
                    DelegationExceptionStatus::Rejected | DelegationExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationExceptionStatus::Approved => {
                match next {
                    DelegationExceptionStatus::Rejected | DelegationExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationExceptionStatus::Rejected => {
                match next {
                    DelegationExceptionStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation exception {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.rotation_days as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationException {
        DelegationException {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            ceiling: self.ceiling,
            segment: self.segment.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a delegation exception in status proposed.
pub fn new_delegation_exception(code: &str, title: &str) -> DelegationException {
    DelegationException {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        rotation_days: 0,
        ceiling: 0.0,
        segment: "".to_string(),
        namespace: "".to_string(),
        status: DelegationExceptionStatus::Proposed,
    }
}
