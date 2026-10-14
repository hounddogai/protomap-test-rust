use super::*;

use acme_kit::text;

/// Where a seasonal timeline digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTimelineDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTimelineDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTimelineDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal timeline digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineDigest {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub attempts: i64,
    pub label: String,
    pub summary: String,
    pub namespace: String,
    pub amount: f64,
    pub title: String,
    pub status: SeasonalTimelineDigestStatus,
}

impl SeasonalTimelineDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTimelineDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTimelineDigestStatus) -> bool {
        match self.status {
            SeasonalTimelineDigestStatus::Proposed => {
                return next == SeasonalTimelineDigestStatus::Withdrawn;
            }
            SeasonalTimelineDigestStatus::Approved => {
                match next {
                    SeasonalTimelineDigestStatus::Rejected | SeasonalTimelineDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTimelineDigestStatus::Rejected => {
                return next == SeasonalTimelineDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTimelineDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal timeline digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.attempts as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTimelineDigest {
        SeasonalTimelineDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            attempts: self.attempts,
            label: self.label.clone(),
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            amount: self.amount,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a seasonal timeline digest in status proposed.
pub fn new_seasonal_timeline_digest(code: &str, name: &str) -> SeasonalTimelineDigest {
    SeasonalTimelineDigest {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        attempts: 0,
        label: "".to_string(),
        summary: "".to_string(),
        namespace: "".to_string(),
        amount: 0.0,
        title: "".to_string(),
        status: SeasonalTimelineDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_timeline_digest("code-879", "name-25");
        assert_eq!(item.status, SeasonalTimelineDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_seasonal_timeline_digest("code-879", "name-25");
        if item.move_to(SeasonalTimelineDigestStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTimelineDigestStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_timeline_digest("code-879", "name-25");
        item.status = SeasonalTimelineDigestStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalTimelineDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
