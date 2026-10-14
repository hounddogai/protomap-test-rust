use super::*;

use acme_kit::text;

/// Where a strict timeline group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTimelineGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictTimelineGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictTimelineGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict timeline group as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroup {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub burst: i64,
    pub visible: bool,
    pub host: String,
    pub version: i64,
    pub country: String,
    pub status: StrictTimelineGroupStatus,
}

impl StrictTimelineGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTimelineGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTimelineGroupStatus) -> bool {
        match self.status {
            StrictTimelineGroupStatus::Proposed => {
                return next == StrictTimelineGroupStatus::Withdrawn;
            }
            StrictTimelineGroupStatus::Approved => {
                match next {
                    StrictTimelineGroupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTimelineGroupStatus::Rejected => {
                match next {
                    StrictTimelineGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: StrictTimelineGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict timeline group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTimelineGroup {
        StrictTimelineGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            burst: self.burst,
            visible: self.visible,
            host: self.host.clone(),
            version: self.version,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict timeline group in status proposed.
pub fn new_strict_timeline_group(code: &str, label: &str) -> StrictTimelineGroup {
    StrictTimelineGroup {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        burst: 0,
        visible: false,
        host: "".to_string(),
        version: 0,
        country: "".to_string(),
        status: StrictTimelineGroupStatus::Proposed,
    }
}
