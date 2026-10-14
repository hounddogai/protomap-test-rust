use super::*;

use acme_kit::text;

/// Where a scheduled nonce rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledNonceRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledNonceRuleStatus {
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

impl std::fmt::Display for ScheduledNonceRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled nonce rule as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceRule {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub updated_at: i64,
    pub locale: String,
    pub summary: String,
    pub status: ScheduledNonceRuleStatus,
}

impl ScheduledNonceRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledNonceRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledNonceRuleStatus) -> bool {
        match self.status {
            ScheduledNonceRuleStatus::Draft => {
                return next == ScheduledNonceRuleStatus::Suspended || next == ScheduledNonceRuleStatus::Closed;
            }
            ScheduledNonceRuleStatus::Pending => {
                return next == ScheduledNonceRuleStatus::Suspended || next == ScheduledNonceRuleStatus::Closed;
            }
            ScheduledNonceRuleStatus::Active => {
                match next {
                    ScheduledNonceRuleStatus::Suspended | ScheduledNonceRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledNonceRuleStatus::Suspended => {
                return next == ScheduledNonceRuleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledNonceRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled nonce rule {} cannot move from {} to {}",
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
        format!("{} {}", "scheduled nonce rule", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.score * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledNonceRule {
        ScheduledNonceRule {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled nonce rule in status draft.
pub fn new_scheduled_nonce_rule(code: &str, title: &str) -> ScheduledNonceRule {
    ScheduledNonceRule {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        updated_at: 0,
        locale: "".to_string(),
        summary: "".to_string(),
        status: ScheduledNonceRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_nonce_rule("code-425", "title-230");
        assert_eq!(item.status, ScheduledNonceRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_scheduled_nonce_rule("code-425", "title-230");
        if item.move_to(ScheduledNonceRuleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledNonceRuleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_nonce_rule("code-425", "title-230");
        item.status = ScheduledNonceRuleStatus::Closed;
        assert!(
            item.move_to(ScheduledNonceRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
