use super::*;

use acme_kit::text;

/// Where a primary health check rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryHealthCheckRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryHealthCheckRuleStatus {
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

impl std::fmt::Display for PrimaryHealthCheckRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary health check rule as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRule {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub phone: String,
    pub reviewer: String,
    pub cipher: String,
    pub tenant_id: String,
    pub key_id: String,
    pub owner_id: String,
    pub status: PrimaryHealthCheckRuleStatus,
}

impl PrimaryHealthCheckRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryHealthCheckRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryHealthCheckRuleStatus) -> bool {
        match self.status {
            PrimaryHealthCheckRuleStatus::Planned => {
                match next {
                    PrimaryHealthCheckRuleStatus::Scheduled | PrimaryHealthCheckRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryHealthCheckRuleStatus::Scheduled => {
                match next {
                    PrimaryHealthCheckRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryHealthCheckRuleStatus::InProgress => {
                match next {
                    PrimaryHealthCheckRuleStatus::Completed | PrimaryHealthCheckRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryHealthCheckRuleStatus::Completed => {
                match next {
                    PrimaryHealthCheckRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryHealthCheckRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary health check rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> PrimaryHealthCheckRule {
        PrimaryHealthCheckRule {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary health check rule in status planned.
pub fn new_primary_health_check_rule(code: &str, name: &str) -> PrimaryHealthCheckRule {
    PrimaryHealthCheckRule {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        phone: "".to_string(),
        reviewer: "".to_string(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        key_id: "".to_string(),
        owner_id: "".to_string(),
        status: PrimaryHealthCheckRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_health_check_rule("code-710", "name-185");
        assert_eq!(item.status, PrimaryHealthCheckRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_health_check_rule("code-710", "name-185");
        if item.move_to(PrimaryHealthCheckRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryHealthCheckRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_health_check_rule("code-710", "name-185");
        item.status = PrimaryHealthCheckRuleStatus::Cancelled;
        assert!(
            item.move_to(PrimaryHealthCheckRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
