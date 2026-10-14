use super::*;

use acme_kit::text;

/// Where a regional federation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFederationAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalFederationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RegionalFederationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional federation adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustment {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub reviewer: String,
    pub ends_at: i64,
    pub tenant_id: String,
    pub revision: i64,
    pub status: RegionalFederationAdjustmentStatus,
}

impl RegionalFederationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFederationAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFederationAdjustmentStatus) -> bool {
        match self.status {
            RegionalFederationAdjustmentStatus::Planned => {
                match next {
                    RegionalFederationAdjustmentStatus::Scheduled | RegionalFederationAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFederationAdjustmentStatus::Scheduled => {
                match next {
                    RegionalFederationAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFederationAdjustmentStatus::InProgress => {
                return next == RegionalFederationAdjustmentStatus::Completed
                    || next == RegionalFederationAdjustmentStatus::Cancelled;
            }
            RegionalFederationAdjustmentStatus::Completed => {
                match next {
                    RegionalFederationAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalFederationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional federation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional federation adjustment", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFederationAdjustment {
        RegionalFederationAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            reviewer: self.reviewer.clone(),
            ends_at: self.ends_at,
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional federation adjustment in status planned.
pub fn new_regional_federation_adjustment(code: &str, label: &str) -> RegionalFederationAdjustment {
    RegionalFederationAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        reviewer: "".to_string(),
        ends_at: 0,
        tenant_id: "".to_string(),
        revision: 0,
        status: RegionalFederationAdjustmentStatus::Planned,
    }
}
