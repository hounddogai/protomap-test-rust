use super::*;

use acme_kit::text;

/// Where a timeline score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TimelineScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TimelineScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScore {
    pub code: String,
    pub title: String,
    pub category: String,
    pub region: String,
    pub phone: String,
    pub scope: String,
    pub owner_id: String,
    pub label: String,
    pub sequence: i64,
    pub retries: i64,
    pub status: TimelineScoreStatus,
}

impl TimelineScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineScoreStatus) -> bool {
        match self.status {
            TimelineScoreStatus::Requested => {
                return next == TimelineScoreStatus::Revoked;
            }
            TimelineScoreStatus::Granted => {
                match next {
                    TimelineScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineScoreStatus::Revoked => {
                return next == TimelineScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "timeline score", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.retries as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineScore {
        TimelineScore {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            region: self.region.clone(),
            phone: self.phone.clone(),
            scope: self.scope.clone(),
            owner_id: self.owner_id.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a timeline score in status requested.
pub fn new_timeline_score(code: &str, title: &str) -> TimelineScore {
    TimelineScore {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        region: "".to_string(),
        phone: "".to_string(),
        scope: "".to_string(),
        owner_id: "".to_string(),
        label: "".to_string(),
        sequence: 0,
        retries: 0,
        status: TimelineScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_timeline_score("code-119", "title-823");
        assert_eq!(item.status, TimelineScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_timeline_score("code-119", "title-823");
        if item.move_to(TimelineScoreStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineScoreStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_timeline_score("code-119", "title-823");
        item.status = TimelineScoreStatus::Expired;
        assert!(
            item.move_to(TimelineScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
