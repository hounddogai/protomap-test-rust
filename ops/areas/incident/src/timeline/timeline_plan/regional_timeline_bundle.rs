use super::*;

use acme_kit::text;

/// Where a regional timeline bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTimelineBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalTimelineBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalTimelineBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional timeline bundle as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineBundle {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub principal: String,
    pub description: String,
    pub expires_at: i64,
    pub ratio: f64,
    pub ceiling: f64,
    pub starts_at: i64,
    pub version: i64,
    pub maximum: f64,
    pub status: RegionalTimelineBundleStatus,
}

impl RegionalTimelineBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTimelineBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTimelineBundleStatus) -> bool {
        match self.status {
            RegionalTimelineBundleStatus::Open => {
                match next {
                    RegionalTimelineBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTimelineBundleStatus::InReview => {
                return next == RegionalTimelineBundleStatus::Archived;
            }
            RegionalTimelineBundleStatus::Resolved => {
                return next == RegionalTimelineBundleStatus::Reopened
                    || next == RegionalTimelineBundleStatus::Archived;
            }
            RegionalTimelineBundleStatus::Reopened => {
                match next {
                    RegionalTimelineBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalTimelineBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional timeline bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTimelineBundle {
        RegionalTimelineBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            principal: self.principal.clone(),
            description: self.description.clone(),
            expires_at: self.expires_at,
            ratio: self.ratio,
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            version: self.version,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional timeline bundle in status open.
pub fn new_regional_timeline_bundle(code: &str, name: &str) -> RegionalTimelineBundle {
    RegionalTimelineBundle {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        principal: "".to_string(),
        description: "".to_string(),
        expires_at: 0,
        ratio: 0.0,
        ceiling: 0.0,
        starts_at: 0,
        version: 0,
        maximum: 0.0,
        status: RegionalTimelineBundleStatus::Open,
    }
}
