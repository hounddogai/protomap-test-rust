use super::*;

use acme_kit::text;

/// Where a strict dashboard digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDashboardDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictDashboardDigestStatus {
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

impl std::fmt::Display for StrictDashboardDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict dashboard digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigest {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub floor_value: f64,
    pub owner_id: String,
    pub burst: i64,
    pub rate: f64,
    pub principal: String,
    pub issuer: String,
    pub status: StrictDashboardDigestStatus,
}

impl StrictDashboardDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDashboardDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDashboardDigestStatus) -> bool {
        match self.status {
            StrictDashboardDigestStatus::Planned => {
                match next {
                    StrictDashboardDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDashboardDigestStatus::Scheduled => {
                return next == StrictDashboardDigestStatus::Completed
                    || next == StrictDashboardDigestStatus::Cancelled;
            }
            StrictDashboardDigestStatus::InProgress => {
                match next {
                    StrictDashboardDigestStatus::Completed | StrictDashboardDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDashboardDigestStatus::Completed => {
                return next == StrictDashboardDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDashboardDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict dashboard digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.rate + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDashboardDigest {
        StrictDashboardDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            owner_id: self.owner_id.clone(),
            burst: self.burst,
            rate: self.rate,
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict dashboard digest in status planned.
pub fn new_strict_dashboard_digest(code: &str, title: &str) -> StrictDashboardDigest {
    StrictDashboardDigest {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        floor_value: 0.0,
        owner_id: "".to_string(),
        burst: 0,
        rate: 0.0,
        principal: "".to_string(),
        issuer: "".to_string(),
        status: StrictDashboardDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_dashboard_digest("code-694", "title-582");
        assert_eq!(item.status, StrictDashboardDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_strict_dashboard_digest("code-694", "title-582");
        if item.move_to(StrictDashboardDigestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictDashboardDigestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_dashboard_digest("code-694", "title-582");
        item.status = StrictDashboardDigestStatus::Cancelled;
        assert!(
            item.move_to(StrictDashboardDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
