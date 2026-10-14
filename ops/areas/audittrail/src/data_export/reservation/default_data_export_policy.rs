use super::*;

use acme_kit::text;

/// Where a default data export policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDataExportPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultDataExportPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultDataExportPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default data export policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicy {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub scope: String,
    pub label: String,
    pub issuer: String,
    pub amount: f64,
    pub status: DefaultDataExportPolicyStatus,
}

impl DefaultDataExportPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDataExportPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDataExportPolicyStatus) -> bool {
        match self.status {
            DefaultDataExportPolicyStatus::Requested => {
                match next {
                    DefaultDataExportPolicyStatus::Granted | DefaultDataExportPolicyStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportPolicyStatus::Granted => {
                return next == DefaultDataExportPolicyStatus::Revoked
                    || next == DefaultDataExportPolicyStatus::Expired;
            }
            DefaultDataExportPolicyStatus::Revoked => {
                match next {
                    DefaultDataExportPolicyStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultDataExportPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default data export policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> DefaultDataExportPolicy {
        DefaultDataExportPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default data export policy in status requested.
pub fn new_default_data_export_policy(code: &str, title: &str) -> DefaultDataExportPolicy {
    DefaultDataExportPolicy {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        scope: "".to_string(),
        label: "".to_string(),
        issuer: "".to_string(),
        amount: 0.0,
        status: DefaultDataExportPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_data_export_policy("code-982", "title-460");
        assert_eq!(item.status, DefaultDataExportPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_data_export_policy("code-982", "title-460");
        if item.move_to(DefaultDataExportPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDataExportPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_data_export_policy("code-982", "title-460");
        item.status = DefaultDataExportPolicyStatus::Expired;
        assert!(
            item.move_to(DefaultDataExportPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
