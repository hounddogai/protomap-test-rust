use super::*;

use acme_kit::text;

/// Where a seasonal failover rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFailoverRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalFailoverRuleStatus {
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

impl std::fmt::Display for SeasonalFailoverRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal failover rule as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRule {
    pub code: String,
    pub name: String,
    pub description: String,
    pub attempts: i64,
    pub fingerprint: String,
    pub label: String,
    pub status: SeasonalFailoverRuleStatus,
}

impl SeasonalFailoverRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFailoverRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFailoverRuleStatus) -> bool {
        match self.status {
            SeasonalFailoverRuleStatus::Draft => {
                match next {
                    SeasonalFailoverRuleStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFailoverRuleStatus::Pending => {
                match next {
                    SeasonalFailoverRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFailoverRuleStatus::Active => {
                match next {
                    SeasonalFailoverRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFailoverRuleStatus::Suspended => {
                match next {
                    SeasonalFailoverRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalFailoverRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal failover rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> SeasonalFailoverRule {
        SeasonalFailoverRule {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal failover rule in status draft.
pub fn new_seasonal_failover_rule(code: &str, name: &str) -> SeasonalFailoverRule {
    SeasonalFailoverRule {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        attempts: 0,
        fingerprint: "".to_string(),
        label: "".to_string(),
        status: SeasonalFailoverRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_failover_rule("code-672", "name-475");
        assert_eq!(item.status, SeasonalFailoverRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_seasonal_failover_rule("code-672", "name-475");
        if item.move_to(SeasonalFailoverRuleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFailoverRuleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_failover_rule("code-672", "name-475");
        item.status = SeasonalFailoverRuleStatus::Closed;
        assert!(
            item.move_to(SeasonalFailoverRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
