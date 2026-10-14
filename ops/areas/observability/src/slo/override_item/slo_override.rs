use super::*;

use acme_kit::text;

/// Where a slo override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SloOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SloOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo override as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloOverride {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub summary: String,
    pub burst: i64,
    pub revision: i64,
    pub window_days: i64,
    pub status: SloOverrideStatus,
}

impl SloOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloOverrideStatus) -> bool {
        match self.status {
            SloOverrideStatus::Proposed => {
                match next {
                    SloOverrideStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloOverrideStatus::Approved => {
                return next == SloOverrideStatus::Rejected || next == SloOverrideStatus::Withdrawn;
            }
            SloOverrideStatus::Rejected => {
                match next {
                    SloOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SloOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.revision as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloOverride {
        SloOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            summary: self.summary.clone(),
            burst: self.burst,
            revision: self.revision,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo override in status proposed.
pub fn new_slo_override(code: &str, name: &str) -> SloOverride {
    SloOverride {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        summary: "".to_string(),
        burst: 0,
        revision: 0,
        window_days: 0,
        status: SloOverrideStatus::Proposed,
    }
}
