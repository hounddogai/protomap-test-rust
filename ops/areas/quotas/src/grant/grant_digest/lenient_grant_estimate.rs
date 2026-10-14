use super::*;

use acme_kit::text;

/// Where a lenient grant estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientGrantEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientGrantEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient grant estimate as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimate {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub timezone: String,
    pub archived: bool,
    pub sequence: i64,
    pub active: bool,
    pub name: String,
    pub status: LenientGrantEstimateStatus,
}

impl LenientGrantEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantEstimateStatus) -> bool {
        match self.status {
            LenientGrantEstimateStatus::Created => {
                return next == LenientGrantEstimateStatus::Retired;
            }
            LenientGrantEstimateStatus::Validated => {
                return next == LenientGrantEstimateStatus::Published;
            }
            LenientGrantEstimateStatus::Published => {
                return next == LenientGrantEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientGrantEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient grant estimate", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantEstimate {
        LenientGrantEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            timezone: self.timezone.clone(),
            archived: self.archived,
            sequence: self.sequence,
            active: self.active,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient grant estimate in status created.
pub fn new_lenient_grant_estimate(code: &str, label: &str) -> LenientGrantEstimate {
    LenientGrantEstimate {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        timezone: "".to_string(),
        archived: false,
        sequence: 0,
        active: false,
        name: "".to_string(),
        status: LenientGrantEstimateStatus::Created,
    }
}
