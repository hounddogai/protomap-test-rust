use super::*;

use acme_kit::text;

/// Where a scheduled identity rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIdentityRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledIdentityRuleStatus {
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

impl std::fmt::Display for ScheduledIdentityRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled identity rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityRule {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub checksum: String,
    pub owner_id: String,
    pub margin: f64,
    pub principal: String,
    pub tenant_id: String,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub status: ScheduledIdentityRuleStatus,
}

impl ScheduledIdentityRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIdentityRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIdentityRuleStatus) -> bool {
        match self.status {
            ScheduledIdentityRuleStatus::New => {
                match next {
                    ScheduledIdentityRuleStatus::Running | ScheduledIdentityRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityRuleStatus::Queued => {
                match next {
                    ScheduledIdentityRuleStatus::Running | ScheduledIdentityRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityRuleStatus::Running => {
                return next == ScheduledIdentityRuleStatus::Done;
            }
            ScheduledIdentityRuleStatus::Done => {
                match next {
                    ScheduledIdentityRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledIdentityRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled identity rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledIdentityRule {
        ScheduledIdentityRule {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            margin: self.margin,
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled identity rule in status new.
pub fn new_scheduled_identity_rule(code: &str, title: &str) -> ScheduledIdentityRule {
    ScheduledIdentityRule {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        checksum: "".to_string(),
        owner_id: "".to_string(),
        margin: 0.0,
        principal: "".to_string(),
        tenant_id: "".to_string(),
        maximum: 0.0,
        duration_minutes: 0,
        burst: 0,
        status: ScheduledIdentityRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_identity_rule("code-257", "title-157");
        assert_eq!(item.status, ScheduledIdentityRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_scheduled_identity_rule("code-257", "title-157");
        if item.move_to(ScheduledIdentityRuleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledIdentityRuleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_identity_rule("code-257", "title-157");
        item.status = ScheduledIdentityRuleStatus::Failed;
        assert!(
            item.move_to(ScheduledIdentityRuleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
