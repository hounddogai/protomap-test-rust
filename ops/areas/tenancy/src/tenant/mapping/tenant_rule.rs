use super::*;

use acme_kit::text;

/// Where a tenant rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TenantRuleStatus {
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

impl std::fmt::Display for TenantRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRule {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub limit: i64,
    pub timezone: String,
    pub phone: String,
    pub minimum: f64,
    pub capacity: i64,
    pub status: TenantRuleStatus,
}

impl TenantRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantRuleStatus) -> bool {
        match self.status {
            TenantRuleStatus::Planned => {
                match next {
                    TenantRuleStatus::Scheduled | TenantRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantRuleStatus::Scheduled => {
                return next == TenantRuleStatus::Cancelled;
            }
            TenantRuleStatus::InProgress => {
                match next {
                    TenantRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantRuleStatus::Completed => {
                return next == TenantRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.floor_value - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantRule {
        TenantRule {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            timezone: self.timezone.clone(),
            phone: self.phone.clone(),
            minimum: self.minimum,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tenant rule in status planned.
pub fn new_tenant_rule(code: &str, name: &str) -> TenantRule {
    TenantRule {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        limit: 0,
        timezone: "".to_string(),
        phone: "".to_string(),
        minimum: 0.0,
        capacity: 0,
        status: TenantRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tenant_rule("code-743", "name-528");
        assert_eq!(item.status, TenantRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tenant_rule("code-743", "name-528");
        if item.move_to(TenantRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tenant_rule("code-743", "name-528");
        item.status = TenantRuleStatus::Cancelled;
        assert!(
            item.move_to(TenantRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
