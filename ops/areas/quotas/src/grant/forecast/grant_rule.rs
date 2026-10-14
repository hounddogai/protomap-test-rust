use super::*;

use acme_kit::text;

/// Where a grant rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GrantRuleStatus {
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

impl std::fmt::Display for GrantRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant rule as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRule {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub window_days: i64,
    pub weight: f64,
    pub grace_days: i64,
    pub owner_id: String,
    pub tenant_id: String,
    pub archived: bool,
    pub segment: String,
    pub status: GrantRuleStatus,
}

impl GrantRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantRuleStatus) -> bool {
        match self.status {
            GrantRuleStatus::Draft => {
                return next == GrantRuleStatus::Suspended;
            }
            GrantRuleStatus::Pending => {
                match next {
                    GrantRuleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRuleStatus::Active => {
                return next == GrantRuleStatus::Suspended || next == GrantRuleStatus::Closed;
            }
            GrantRuleStatus::Suspended => {
                return next == GrantRuleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.window_days as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantRule {
        GrantRule {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            window_days: self.window_days,
            weight: self.weight,
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a grant rule in status draft.
pub fn new_grant_rule(code: &str, title: &str) -> GrantRule {
    GrantRule {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        window_days: 0,
        weight: 0.0,
        grace_days: 0,
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        archived: false,
        segment: "".to_string(),
        status: GrantRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_grant_rule("code-922", "title-497");
        assert_eq!(item.status, GrantRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_grant_rule("code-922", "title-497");
        if item.move_to(GrantRuleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantRuleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_grant_rule("code-922", "title-497");
        item.status = GrantRuleStatus::Closed;
        assert!(item.move_to(GrantRuleStatus::Draft).is_err(), "expected move to fail");
    }
}
