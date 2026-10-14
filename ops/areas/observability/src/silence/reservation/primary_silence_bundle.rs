use super::*;

use acme_kit::text;

/// Where a primary silence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySilenceBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimarySilenceBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimarySilenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary silence bundle as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundle {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub summary: String,
    pub timezone: String,
    pub ceiling: f64,
    pub status: PrimarySilenceBundleStatus,
}

impl PrimarySilenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySilenceBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySilenceBundleStatus) -> bool {
        match self.status {
            PrimarySilenceBundleStatus::Proposed => {
                return next == PrimarySilenceBundleStatus::Rejected;
            }
            PrimarySilenceBundleStatus::Approved => {
                match next {
                    PrimarySilenceBundleStatus::Rejected | PrimarySilenceBundleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySilenceBundleStatus::Rejected => {
                return next == PrimarySilenceBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySilenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary silence bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary silence bundle", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.version as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySilenceBundle {
        PrimarySilenceBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary silence bundle in status proposed.
pub fn new_primary_silence_bundle(code: &str, name: &str) -> PrimarySilenceBundle {
    PrimarySilenceBundle {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        summary: "".to_string(),
        timezone: "".to_string(),
        ceiling: 0.0,
        status: PrimarySilenceBundleStatus::Proposed,
    }
}
