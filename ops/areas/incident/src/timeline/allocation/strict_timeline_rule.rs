use super::*;

use acme_kit::text;

/// Where a strict timeline rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTimelineRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictTimelineRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictTimelineRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict timeline rule as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineRule {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub timezone: String,
    pub description: String,
    pub score: f64,
    pub status: StrictTimelineRuleStatus,
}

impl StrictTimelineRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTimelineRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTimelineRuleStatus) -> bool {
        match self.status {
            StrictTimelineRuleStatus::Draft => {
                match next {
                    StrictTimelineRuleStatus::Active | StrictTimelineRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTimelineRuleStatus::Pending => {
                match next {
                    StrictTimelineRuleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTimelineRuleStatus::Active => {
                return next == StrictTimelineRuleStatus::Closed;
            }
            StrictTimelineRuleStatus::Suspended => {
                return next == StrictTimelineRuleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictTimelineRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict timeline rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> StrictTimelineRule {
        StrictTimelineRule {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            timezone: self.timezone.clone(),
            description: self.description.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict timeline rule in status draft.
pub fn new_strict_timeline_rule(code: &str, name: &str) -> StrictTimelineRule {
    StrictTimelineRule {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        timezone: "".to_string(),
        description: "".to_string(),
        score: 0.0,
        status: StrictTimelineRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_timeline_rule("code-775", "name-639");
        assert_eq!(item.status, StrictTimelineRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_timeline_rule("code-775", "name-639");
        if item.move_to(StrictTimelineRuleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTimelineRuleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_timeline_rule("code-775", "name-639");
        item.status = StrictTimelineRuleStatus::Closed;
        assert!(
            item.move_to(StrictTimelineRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
