use super::*;

use acme_kit::text;

/// Where a primary release claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReleaseClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryReleaseClaimStatus {
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

impl std::fmt::Display for PrimaryReleaseClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary release claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaim {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub unit: String,
    pub ratio: f64,
    pub status: PrimaryReleaseClaimStatus,
}

impl PrimaryReleaseClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReleaseClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReleaseClaimStatus) -> bool {
        match self.status {
            PrimaryReleaseClaimStatus::Planned => {
                match next {
                    PrimaryReleaseClaimStatus::Scheduled | PrimaryReleaseClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReleaseClaimStatus::Scheduled => {
                return next == PrimaryReleaseClaimStatus::Completed || next == PrimaryReleaseClaimStatus::Cancelled;
            }
            PrimaryReleaseClaimStatus::InProgress => {
                return next == PrimaryReleaseClaimStatus::Cancelled;
            }
            PrimaryReleaseClaimStatus::Completed => {
                return next == PrimaryReleaseClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryReleaseClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary release claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        (self.ratio * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReleaseClaim {
        PrimaryReleaseClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            unit: self.unit.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary release claim in status planned.
pub fn new_primary_release_claim(code: &str, title: &str) -> PrimaryReleaseClaim {
    PrimaryReleaseClaim {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        unit: "".to_string(),
        ratio: 0.0,
        status: PrimaryReleaseClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_release_claim("code-867", "title-398");
        assert_eq!(item.status, PrimaryReleaseClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_release_claim("code-867", "title-398");
        if item.move_to(PrimaryReleaseClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReleaseClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_release_claim("code-867", "title-398");
        item.status = PrimaryReleaseClaimStatus::Cancelled;
        assert!(
            item.move_to(PrimaryReleaseClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
