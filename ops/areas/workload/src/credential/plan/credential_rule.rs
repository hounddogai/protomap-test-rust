use super::*;

use acme_kit::text;

/// Where a credential rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CredentialRuleStatus {
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

impl std::fmt::Display for CredentialRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential rule as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRule {
    pub code: String,
    pub title: String,
    pub label: String,
    pub approved: bool,
    pub attempts: i64,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub limit: i64,
    pub description: String,
    pub status: CredentialRuleStatus,
}

impl CredentialRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialRuleStatus) -> bool {
        match self.status {
            CredentialRuleStatus::Draft => {
                match next {
                    CredentialRuleStatus::Active | CredentialRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialRuleStatus::Pending => {
                match next {
                    CredentialRuleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialRuleStatus::Active => {
                return next == CredentialRuleStatus::Suspended;
            }
            CredentialRuleStatus::Suspended => {
                match next {
                    CredentialRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: CredentialRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.limit as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialRule {
        CredentialRule {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            approved: self.approved,
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential rule in status draft.
pub fn new_credential_rule(code: &str, title: &str) -> CredentialRule {
    CredentialRule {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        approved: false,
        attempts: 0,
        contact_email: "".to_string(),
        duration_minutes: 0,
        limit: 0,
        description: "".to_string(),
        status: CredentialRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_credential_rule("code-402", "title-876");
        assert_eq!(item.status, CredentialRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_credential_rule("code-402", "title-876");
        if item.move_to(CredentialRuleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialRuleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_credential_rule("code-402", "title-876");
        item.status = CredentialRuleStatus::Closed;
        assert!(
            item.move_to(CredentialRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
