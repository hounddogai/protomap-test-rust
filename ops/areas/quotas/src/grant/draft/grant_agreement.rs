use super::*;

use acme_kit::text;

/// Where a grant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant agreement as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAgreement {
    pub code: String,
    pub label: String,
    pub region: String,
    pub window_days: i64,
    pub sequence: i64,
    pub status: GrantAgreementStatus,
}

impl GrantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantAgreementStatus) -> bool {
        match self.status {
            GrantAgreementStatus::Requested => {
                return next == GrantAgreementStatus::Granted || next == GrantAgreementStatus::Expired;
            }
            GrantAgreementStatus::Granted => {
                match next {
                    GrantAgreementStatus::Revoked | GrantAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAgreementStatus::Revoked => {
                match next {
                    GrantAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: GrantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant agreement", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantAgreement {
        GrantAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            window_days: self.window_days,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant agreement in status requested.
pub fn new_grant_agreement(code: &str, label: &str) -> GrantAgreement {
    GrantAgreement {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        window_days: 0,
        sequence: 0,
        status: GrantAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_agreement("code-177", "label-519");
        assert_eq!(item.status, GrantAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_grant_agreement("code-177", "label-519");
        if item.move_to(GrantAgreementStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantAgreementStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_agreement("code-177", "label-519");
        item.status = GrantAgreementStatus::Expired;
        assert!(
            item.move_to(GrantAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
