use super::*;

use acme_kit::text;

/// Where a adaptive identity target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIdentityTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveIdentityTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveIdentityTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive identity target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityTarget {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub capacity: i64,
    pub segment: String,
    pub title: String,
    pub maximum: f64,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub status: AdaptiveIdentityTargetStatus,
}

impl AdaptiveIdentityTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIdentityTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIdentityTargetStatus) -> bool {
        match self.status {
            AdaptiveIdentityTargetStatus::New => {
                return next == AdaptiveIdentityTargetStatus::Queued;
            }
            AdaptiveIdentityTargetStatus::Queued => {
                match next {
                    AdaptiveIdentityTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIdentityTargetStatus::Running => {
                match next {
                    AdaptiveIdentityTargetStatus::Done | AdaptiveIdentityTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIdentityTargetStatus::Done => {
                match next {
                    AdaptiveIdentityTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveIdentityTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive identity target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.created_at as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveIdentityTarget {
        AdaptiveIdentityTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            capacity: self.capacity,
            segment: self.segment.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive identity target in status new.
pub fn new_adaptive_identity_target(code: &str, name: &str) -> AdaptiveIdentityTarget {
    AdaptiveIdentityTarget {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        capacity: 0,
        segment: "".to_string(),
        title: "".to_string(),
        maximum: 0.0,
        created_at: 0,
        duration_minutes: 0,
        status: AdaptiveIdentityTargetStatus::New,
    }
}
