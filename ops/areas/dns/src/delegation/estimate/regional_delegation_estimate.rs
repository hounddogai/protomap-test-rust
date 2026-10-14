use super::*;

use acme_kit::text;

/// Where a regional delegation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDelegationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalDelegationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalDelegationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional delegation estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationEstimate {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub created_at: i64,
    pub principal: String,
    pub duration_minutes: i64,
    pub visible: bool,
    pub status: RegionalDelegationEstimateStatus,
}

impl RegionalDelegationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDelegationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDelegationEstimateStatus) -> bool {
        match self.status {
            RegionalDelegationEstimateStatus::Created => {
                return next == RegionalDelegationEstimateStatus::Published;
            }
            RegionalDelegationEstimateStatus::Validated => {
                match next {
                    RegionalDelegationEstimateStatus::Published | RegionalDelegationEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDelegationEstimateStatus::Published => {
                match next {
                    RegionalDelegationEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionalDelegationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional delegation estimate {} cannot move from {} to {}",
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
        format!("{} {}", "regional delegation estimate", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDelegationEstimate {
        RegionalDelegationEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            created_at: self.created_at,
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional delegation estimate in status created.
pub fn new_regional_delegation_estimate(code: &str, title: &str) -> RegionalDelegationEstimate {
    RegionalDelegationEstimate {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        created_at: 0,
        principal: "".to_string(),
        duration_minutes: 0,
        visible: false,
        status: RegionalDelegationEstimateStatus::Created,
    }
}
