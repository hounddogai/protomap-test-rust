use super::*;

use acme_kit::text;

/// Where a identity claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for IdentityClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One identity claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaim {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub region: String,
    pub principal: String,
    pub reviewer: String,
    pub summary: String,
    pub status: IdentityClaimStatus,
}

impl IdentityClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityClaimStatus) -> bool {
        match self.status {
            IdentityClaimStatus::Planned => {
                match next {
                    IdentityClaimStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityClaimStatus::Scheduled => {
                match next {
                    IdentityClaimStatus::Completed | IdentityClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityClaimStatus::InProgress => {
                return next == IdentityClaimStatus::Completed;
            }
            IdentityClaimStatus::Completed => {
                return next == IdentityClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity claim {} cannot move from {} to {}",
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
        self.quota_units as f64
    }

    pub fn copy(&self) -> IdentityClaim {
        IdentityClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            principal: self.principal.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a identity claim in status planned.
pub fn new_identity_claim(code: &str, title: &str) -> IdentityClaim {
    IdentityClaim {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        region: "".to_string(),
        principal: "".to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        status: IdentityClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_identity_claim("code-739", "title-660");
        assert_eq!(item.status, IdentityClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_identity_claim("code-739", "title-660");
        if item.move_to(IdentityClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_identity_claim("code-739", "title-660");
        item.status = IdentityClaimStatus::Cancelled;
        assert!(
            item.move_to(IdentityClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
