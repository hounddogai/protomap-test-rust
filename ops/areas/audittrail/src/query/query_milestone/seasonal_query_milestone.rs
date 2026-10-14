use super::*;

use acme_kit::text;

/// Where a seasonal query milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQueryMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalQueryMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalQueryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal query milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestone {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub pinned: bool,
    pub threshold: f64,
    pub country: String,
    pub status: SeasonalQueryMilestoneStatus,
}

impl SeasonalQueryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQueryMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQueryMilestoneStatus) -> bool {
        match self.status {
            SeasonalQueryMilestoneStatus::Requested => {
                match next {
                    SeasonalQueryMilestoneStatus::Granted | SeasonalQueryMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQueryMilestoneStatus::Granted => {
                return next == SeasonalQueryMilestoneStatus::Revoked;
            }
            SeasonalQueryMilestoneStatus::Revoked => {
                return next == SeasonalQueryMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalQueryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal query milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> SeasonalQueryMilestone {
        SeasonalQueryMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            pinned: self.pinned,
            threshold: self.threshold,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal query milestone in status requested.
pub fn new_seasonal_query_milestone(code: &str, title: &str) -> SeasonalQueryMilestone {
    SeasonalQueryMilestone {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        pinned: false,
        threshold: 0.0,
        country: "".to_string(),
        status: SeasonalQueryMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_query_milestone("code-354", "title-648");
        assert_eq!(item.status, SeasonalQueryMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_query_milestone("code-354", "title-648");
        if item.move_to(SeasonalQueryMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalQueryMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_query_milestone("code-354", "title-648");
        item.status = SeasonalQueryMilestoneStatus::Expired;
        assert!(
            item.move_to(SeasonalQueryMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
