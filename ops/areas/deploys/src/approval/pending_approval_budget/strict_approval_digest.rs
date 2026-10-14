use super::*;

use acme_kit::text;

/// Where a strict approval digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictApprovalDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictApprovalDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict approval digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigest {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub unit: String,
    pub limit: i64,
    pub description: String,
    pub score: f64,
    pub weight: f64,
    pub scope: String,
    pub status: StrictApprovalDigestStatus,
}

impl StrictApprovalDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalDigestStatus) -> bool {
        match self.status {
            StrictApprovalDigestStatus::Requested => {
                match next {
                    StrictApprovalDigestStatus::Revoked | StrictApprovalDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalDigestStatus::Granted => {
                return next == StrictApprovalDigestStatus::Expired;
            }
            StrictApprovalDigestStatus::Revoked => {
                return next == StrictApprovalDigestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictApprovalDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict approval digest", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.weight - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictApprovalDigest {
        StrictApprovalDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            limit: self.limit,
            description: self.description.clone(),
            score: self.score,
            weight: self.weight,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict approval digest in status requested.
pub fn new_strict_approval_digest(code: &str, name: &str) -> StrictApprovalDigest {
    StrictApprovalDigest {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        limit: 0,
        description: "".to_string(),
        score: 0.0,
        weight: 0.0,
        scope: "".to_string(),
        status: StrictApprovalDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_approval_digest("code-737", "name-915");
        assert_eq!(item.status, StrictApprovalDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_strict_approval_digest("code-737", "name-915");
        if item.move_to(StrictApprovalDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_approval_digest("code-737", "name-915");
        item.status = StrictApprovalDigestStatus::Expired;
        assert!(
            item.move_to(StrictApprovalDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
