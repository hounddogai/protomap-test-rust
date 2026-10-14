use super::*;

use acme_kit::text;

/// Where a global release agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalReleaseAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalReleaseAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global release agreement as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAgreement {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub name: String,
    pub scope: String,
    pub amount: f64,
    pub status: GlobalReleaseAgreementStatus,
}

impl GlobalReleaseAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseAgreementStatus) -> bool {
        match self.status {
            GlobalReleaseAgreementStatus::Requested => {
                match next {
                    GlobalReleaseAgreementStatus::Revoked | GlobalReleaseAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAgreementStatus::Granted => {
                match next {
                    GlobalReleaseAgreementStatus::Revoked | GlobalReleaseAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAgreementStatus::Revoked => {
                match next {
                    GlobalReleaseAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalReleaseAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> GlobalReleaseAgreement {
        GlobalReleaseAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            name: self.name.clone(),
            scope: self.scope.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global release agreement in status requested.
pub fn new_global_release_agreement(code: &str, label: &str) -> GlobalReleaseAgreement {
    GlobalReleaseAgreement {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        name: "".to_string(),
        scope: "".to_string(),
        amount: 0.0,
        status: GlobalReleaseAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_release_agreement("code-335", "label-274");
        assert_eq!(item.status, GlobalReleaseAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_global_release_agreement("code-335", "label-274");
        if item.move_to(GlobalReleaseAgreementStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReleaseAgreementStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_release_agreement("code-335", "label-274");
        item.status = GlobalReleaseAgreementStatus::Expired;
        assert!(
            item.move_to(GlobalReleaseAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
