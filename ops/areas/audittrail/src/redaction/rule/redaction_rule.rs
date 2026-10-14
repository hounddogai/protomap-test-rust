use super::*;

use acme_kit::text;

/// Where a redaction rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionRuleStatus {
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

impl std::fmt::Display for RedactionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRule {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub country: String,
    pub ttl_seconds: i64,
    pub ends_at: i64,
    pub status: RedactionRuleStatus,
}

impl RedactionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionRuleStatus) -> bool {
        match self.status {
            RedactionRuleStatus::New => {
                match next {
                    RedactionRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionRuleStatus::Queued => {
                return next == RedactionRuleStatus::Running || next == RedactionRuleStatus::Done;
            }
            RedactionRuleStatus::Running => {
                return next == RedactionRuleStatus::Done || next == RedactionRuleStatus::Failed;
            }
            RedactionRuleStatus::Done => {
                match next {
                    RedactionRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: RedactionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionRule {
        RedactionRule {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            country: self.country.clone(),
            ttl_seconds: self.ttl_seconds,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a redaction rule in status new.
pub fn new_redaction_rule(code: &str, title: &str) -> RedactionRule {
    RedactionRule {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        country: "".to_string(),
        ttl_seconds: 0,
        ends_at: 0,
        status: RedactionRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_redaction_rule("code-129", "title-820");
        assert_eq!(item.status, RedactionRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_redaction_rule("code-129", "title-820");
        if item.move_to(RedactionRuleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionRuleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_redaction_rule("code-129", "title-820");
        item.status = RedactionRuleStatus::Failed;
        assert!(item.move_to(RedactionRuleStatus::New).is_err(), "expected move to fail");
    }
}
