use super::*;

use acme_kit::text;

/// Where a sink bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink bundle as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundle {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub country: String,
    pub priority: i64,
    pub summary: String,
    pub status: SinkBundleStatus,
}

impl SinkBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkBundleStatus) -> bool {
        match self.status {
            SinkBundleStatus::Requested => {
                return next == SinkBundleStatus::Granted;
            }
            SinkBundleStatus::Granted => {
                match next {
                    SinkBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkBundleStatus::Revoked => {
                match next {
                    SinkBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: SinkBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.priority as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkBundle {
        SinkBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            country: self.country.clone(),
            priority: self.priority,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a sink bundle in status requested.
pub fn new_sink_bundle(code: &str, name: &str) -> SinkBundle {
    SinkBundle {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        country: "".to_string(),
        priority: 0,
        summary: "".to_string(),
        status: SinkBundleStatus::Requested,
    }
}
