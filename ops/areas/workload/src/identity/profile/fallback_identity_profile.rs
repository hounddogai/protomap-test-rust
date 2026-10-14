use super::*;

use acme_kit::text;

/// Where a fallback identity profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIdentityProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackIdentityProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackIdentityProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback identity profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfile {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub version: i64,
    pub unit: String,
    pub audience: String,
    pub checksum: String,
    pub budget: f64,
    pub notes: String,
    pub status: FallbackIdentityProfileStatus,
}

impl FallbackIdentityProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIdentityProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIdentityProfileStatus) -> bool {
        match self.status {
            FallbackIdentityProfileStatus::Planned => {
                return next == FallbackIdentityProfileStatus::Completed;
            }
            FallbackIdentityProfileStatus::Scheduled => {
                match next {
                    FallbackIdentityProfileStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIdentityProfileStatus::InProgress => {
                match next {
                    FallbackIdentityProfileStatus::Completed | FallbackIdentityProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIdentityProfileStatus::Completed => {
                return next == FallbackIdentityProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackIdentityProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback identity profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        (self.budget * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIdentityProfile {
        FallbackIdentityProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            version: self.version,
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            checksum: self.checksum.clone(),
            budget: self.budget,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback identity profile in status planned.
pub fn new_fallback_identity_profile(code: &str, title: &str) -> FallbackIdentityProfile {
    FallbackIdentityProfile {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        version: 0,
        unit: "".to_string(),
        audience: "".to_string(),
        checksum: "".to_string(),
        budget: 0.0,
        notes: "".to_string(),
        status: FallbackIdentityProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_identity_profile("code-441", "title-796");
        assert_eq!(item.status, FallbackIdentityProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_identity_profile("code-441", "title-796");
        if item.move_to(FallbackIdentityProfileStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackIdentityProfileStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_identity_profile("code-441", "title-796");
        item.status = FallbackIdentityProfileStatus::Cancelled;
        assert!(
            item.move_to(FallbackIdentityProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
