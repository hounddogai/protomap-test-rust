use super::*;

use acme_kit::text;

/// Where a region pin proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin proposal as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposal {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub window_days: i64,
    pub title: String,
    pub version: i64,
    pub description: String,
    pub quota_units: i64,
    pub timezone: String,
    pub visible: bool,
    pub status: RegionPinProposalStatus,
}

impl RegionPinProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinProposalStatus) -> bool {
        match self.status {
            RegionPinProposalStatus::Created => {
                return next == RegionPinProposalStatus::Validated || next == RegionPinProposalStatus::Retired;
            }
            RegionPinProposalStatus::Validated => {
                return next == RegionPinProposalStatus::Retired;
            }
            RegionPinProposalStatus::Published => {
                return next == RegionPinProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinProposal {
        RegionPinProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            window_days: self.window_days,
            title: self.title.clone(),
            version: self.version,
            description: self.description.clone(),
            quota_units: self.quota_units,
            timezone: self.timezone.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin proposal in status created.
pub fn new_region_pin_proposal(code: &str, label: &str) -> RegionPinProposal {
    RegionPinProposal {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        window_days: 0,
        title: "".to_string(),
        version: 0,
        description: "".to_string(),
        quota_units: 0,
        timezone: "".to_string(),
        visible: false,
        status: RegionPinProposalStatus::Created,
    }
}
