use super::*;

use acme_kit::text;

/// Where a release estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReleaseEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReleaseEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release estimate as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEstimate {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub rate: f64,
    pub discount: f64,
    pub visible: bool,
    pub status: ReleaseEstimateStatus,
}

impl ReleaseEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseEstimateStatus) -> bool {
        match self.status {
            ReleaseEstimateStatus::Created => {
                return next == ReleaseEstimateStatus::Published || next == ReleaseEstimateStatus::Retired;
            }
            ReleaseEstimateStatus::Validated => {
                match next {
                    ReleaseEstimateStatus::Published | ReleaseEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseEstimateStatus::Published => {
                match next {
                    ReleaseEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: ReleaseEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.discount * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseEstimate {
        ReleaseEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            rate: self.rate,
            discount: self.discount,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release estimate in status created.
pub fn new_release_estimate(code: &str, name: &str) -> ReleaseEstimate {
    ReleaseEstimate {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        rate: 0.0,
        discount: 0.0,
        visible: false,
        status: ReleaseEstimateStatus::Created,
    }
}
