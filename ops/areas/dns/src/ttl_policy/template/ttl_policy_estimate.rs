use super::*;

use acme_kit::text;

/// Where a ttl policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TtlPolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TtlPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimate {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub discount: f64,
    pub locale: String,
    pub issuer: String,
    pub approved: bool,
    pub archived: bool,
    pub source: String,
    pub status: TtlPolicyEstimateStatus,
}

impl TtlPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyEstimateStatus) -> bool {
        match self.status {
            TtlPolicyEstimateStatus::Requested => {
                match next {
                    TtlPolicyEstimateStatus::Granted | TtlPolicyEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyEstimateStatus::Granted => {
                return next == TtlPolicyEstimateStatus::Revoked;
            }
            TtlPolicyEstimateStatus::Revoked => {
                return next == TtlPolicyEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> TtlPolicyEstimate {
        TtlPolicyEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            discount: self.discount,
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            approved: self.approved,
            archived: self.archived,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a ttl policy estimate in status requested.
pub fn new_ttl_policy_estimate(code: &str, title: &str) -> TtlPolicyEstimate {
    TtlPolicyEstimate {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        discount: 0.0,
        locale: "".to_string(),
        issuer: "".to_string(),
        approved: false,
        archived: false,
        source: "".to_string(),
        status: TtlPolicyEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_ttl_policy_estimate("code-815", "title-278");
        assert_eq!(item.status, TtlPolicyEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_ttl_policy_estimate("code-815", "title-278");
        if item.move_to(TtlPolicyEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_ttl_policy_estimate("code-815", "title-278");
        item.status = TtlPolicyEstimateStatus::Expired;
        assert!(
            item.move_to(TtlPolicyEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
