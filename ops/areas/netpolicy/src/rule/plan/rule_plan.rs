use super::*;

use acme_kit::text;

/// Where a rule plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RulePlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RulePlanStatus {
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

impl std::fmt::Display for RulePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RulePlan {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub issuer: String,
    pub status: RulePlanStatus,
}

impl RulePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RulePlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RulePlanStatus) -> bool {
        match self.status {
            RulePlanStatus::Draft => {
                match next {
                    RulePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RulePlanStatus::Pending => {
                return next == RulePlanStatus::Closed;
            }
            RulePlanStatus::Active => {
                match next {
                    RulePlanStatus::Suspended | RulePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RulePlanStatus::Suspended => {
                match next {
                    RulePlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: RulePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RulePlan {
        RulePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule plan in status draft.
pub fn new_rule_plan(code: &str, title: &str) -> RulePlan {
    RulePlan {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        ttl_seconds: 0,
        issuer: "".to_string(),
        status: RulePlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rule_plan("code-886", "title-147");
        assert_eq!(item.status, RulePlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_rule_plan("code-886", "title-147");
        if item.move_to(RulePlanStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RulePlanStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rule_plan("code-886", "title-147");
        item.status = RulePlanStatus::Closed;
        assert!(item.move_to(RulePlanStatus::Draft).is_err(), "expected move to fail");
    }
}
