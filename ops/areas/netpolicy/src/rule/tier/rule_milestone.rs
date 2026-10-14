use super::*;

use acme_kit::text;

/// Where a rule milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RuleMilestoneStatus {
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

impl std::fmt::Display for RuleMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestone {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub expires_at: i64,
    pub limit: i64,
    pub starts_at: i64,
    pub name: String,
    pub timezone: String,
    pub attempts: i64,
    pub status: RuleMilestoneStatus,
}

impl RuleMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleMilestoneStatus) -> bool {
        match self.status {
            RuleMilestoneStatus::Draft => {
                return next == RuleMilestoneStatus::Pending;
            }
            RuleMilestoneStatus::Pending => {
                match next {
                    RuleMilestoneStatus::Active | RuleMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleMilestoneStatus::Active => {
                return next == RuleMilestoneStatus::Suspended;
            }
            RuleMilestoneStatus::Suspended => {
                match next {
                    RuleMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: RuleMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleMilestone {
        RuleMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            expires_at: self.expires_at,
            limit: self.limit,
            starts_at: self.starts_at,
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rule milestone in status draft.
pub fn new_rule_milestone(code: &str, title: &str) -> RuleMilestone {
    RuleMilestone {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        expires_at: 0,
        limit: 0,
        starts_at: 0,
        name: "".to_string(),
        timezone: "".to_string(),
        attempts: 0,
        status: RuleMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rule_milestone("code-408", "title-453");
        assert_eq!(item.status, RuleMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rule_milestone("code-408", "title-453");
        if item.move_to(RuleMilestoneStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleMilestoneStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rule_milestone("code-408", "title-453");
        item.status = RuleMilestoneStatus::Closed;
        assert!(
            item.move_to(RuleMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
