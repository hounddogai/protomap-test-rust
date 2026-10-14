use super::*;

use acme_kit::text;

/// Where a lenient egress profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientEgressProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientEgressProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient egress profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProfile {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub owner_id: String,
    pub fingerprint: String,
    pub currency: String,
    pub status: LenientEgressProfileStatus,
}

impl LenientEgressProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressProfileStatus) -> bool {
        match self.status {
            LenientEgressProfileStatus::Open => {
                return next == LenientEgressProfileStatus::InReview || next == LenientEgressProfileStatus::Resolved;
            }
            LenientEgressProfileStatus::InReview => {
                match next {
                    LenientEgressProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressProfileStatus::Resolved => {
                match next {
                    LenientEgressProfileStatus::Reopened | LenientEgressProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressProfileStatus::Reopened => {
                return next == LenientEgressProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEgressProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> LenientEgressProfile {
        LenientEgressProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            owner_id: self.owner_id.clone(),
            fingerprint: self.fingerprint.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient egress profile in status open.
pub fn new_lenient_egress_profile(code: &str, name: &str) -> LenientEgressProfile {
    LenientEgressProfile {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        owner_id: "".to_string(),
        fingerprint: "".to_string(),
        currency: "".to_string(),
        status: LenientEgressProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_egress_profile("code-694", "name-131");
        assert_eq!(item.status, LenientEgressProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_egress_profile("code-694", "name-131");
        if item.move_to(LenientEgressProfileStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressProfileStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_egress_profile("code-694", "name-131");
        item.status = LenientEgressProfileStatus::Archived;
        assert!(
            item.move_to(LenientEgressProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
