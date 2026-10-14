use super::*;

use acme_kit::text;

/// Where a scheduled grant rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledGrantRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledGrantRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledGrantRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled grant rule as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRule {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub cipher: String,
    pub attempts: i64,
    pub contact_email: String,
    pub sequence: i64,
    pub status: ScheduledGrantRuleStatus,
}

impl ScheduledGrantRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledGrantRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledGrantRuleStatus) -> bool {
        match self.status {
            ScheduledGrantRuleStatus::Created => {
                return next == ScheduledGrantRuleStatus::Published;
            }
            ScheduledGrantRuleStatus::Validated => {
                match next {
                    ScheduledGrantRuleStatus::Published | ScheduledGrantRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledGrantRuleStatus::Published => {
                return next == ScheduledGrantRuleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledGrantRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled grant rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled grant rule", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledGrantRule {
        ScheduledGrantRule {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled grant rule in status created.
pub fn new_scheduled_grant_rule(code: &str, name: &str) -> ScheduledGrantRule {
    ScheduledGrantRule {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        rotation_days: 0,
        cipher: "".to_string(),
        attempts: 0,
        contact_email: "".to_string(),
        sequence: 0,
        status: ScheduledGrantRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_grant_rule("code-186", "name-840");
        assert_eq!(item.status, ScheduledGrantRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_grant_rule("code-186", "name-840");
        if item.move_to(ScheduledGrantRuleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledGrantRuleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_grant_rule("code-186", "name-840");
        item.status = ScheduledGrantRuleStatus::Retired;
        assert!(
            item.move_to(ScheduledGrantRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
