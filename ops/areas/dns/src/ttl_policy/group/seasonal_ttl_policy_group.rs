use super::*;

use acme_kit::text;

/// Where a seasonal ttl policy group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTtlPolicyGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalTtlPolicyGroupStatus {
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

impl std::fmt::Display for SeasonalTtlPolicyGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal ttl policy group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroup {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub source: String,
    pub window_days: i64,
    pub summary: String,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub duration_minutes: i64,
    pub sequence: i64,
    pub status: SeasonalTtlPolicyGroupStatus,
}

impl SeasonalTtlPolicyGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTtlPolicyGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTtlPolicyGroupStatus) -> bool {
        match self.status {
            SeasonalTtlPolicyGroupStatus::Open => {
                match next {
                    SeasonalTtlPolicyGroupStatus::InReview | SeasonalTtlPolicyGroupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTtlPolicyGroupStatus::InReview => {
                return next == SeasonalTtlPolicyGroupStatus::Resolved
                    || next == SeasonalTtlPolicyGroupStatus::Archived;
            }
            SeasonalTtlPolicyGroupStatus::Resolved => {
                return next == SeasonalTtlPolicyGroupStatus::Reopened
                    || next == SeasonalTtlPolicyGroupStatus::Archived;
            }
            SeasonalTtlPolicyGroupStatus::Reopened => {
                return next == SeasonalTtlPolicyGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTtlPolicyGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal ttl policy group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal ttl policy group", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTtlPolicyGroup {
        SeasonalTtlPolicyGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            source: self.source.clone(),
            window_days: self.window_days,
            summary: self.summary.clone(),
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            duration_minutes: self.duration_minutes,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal ttl policy group in status open.
pub fn new_seasonal_ttl_policy_group(code: &str, title: &str) -> SeasonalTtlPolicyGroup {
    SeasonalTtlPolicyGroup {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        source: "".to_string(),
        window_days: 0,
        summary: "".to_string(),
        maximum: 0.0,
        ttl_seconds: 0,
        priority: 0,
        duration_minutes: 0,
        sequence: 0,
        status: SeasonalTtlPolicyGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_ttl_policy_group("code-286", "title-795");
        assert_eq!(item.status, SeasonalTtlPolicyGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_seasonal_ttl_policy_group("code-286", "title-795");
        if item.move_to(SeasonalTtlPolicyGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTtlPolicyGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_ttl_policy_group("code-286", "title-795");
        item.status = SeasonalTtlPolicyGroupStatus::Archived;
        assert!(
            item.move_to(SeasonalTtlPolicyGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
