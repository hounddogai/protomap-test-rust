use super::*;

use acme_kit::text;

/// Where a peering digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringDigestStatus {
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

impl std::fmt::Display for PeeringDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigest {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub quota_units: i64,
    pub notes: String,
    pub window_days: i64,
    pub status: PeeringDigestStatus,
}

impl PeeringDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringDigestStatus) -> bool {
        match self.status {
            PeeringDigestStatus::Planned => {
                return next == PeeringDigestStatus::InProgress || next == PeeringDigestStatus::Cancelled;
            }
            PeeringDigestStatus::Scheduled => {
                return next == PeeringDigestStatus::Completed;
            }
            PeeringDigestStatus::InProgress => {
                match next {
                    PeeringDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringDigestStatus::Completed => {
                return next == PeeringDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringDigest {
        PeeringDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            quota_units: self.quota_units,
            notes: self.notes.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering digest in status planned.
pub fn new_peering_digest(code: &str, title: &str) -> PeeringDigest {
    PeeringDigest {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        quota_units: 0,
        notes: "".to_string(),
        window_days: 0,
        status: PeeringDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_peering_digest("code-171", "title-169");
        assert_eq!(item.status, PeeringDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_peering_digest("code-171", "title-169");
        if item.move_to(PeeringDigestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringDigestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_peering_digest("code-171", "title-169");
        item.status = PeeringDigestStatus::Cancelled;
        assert!(
            item.move_to(PeeringDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
