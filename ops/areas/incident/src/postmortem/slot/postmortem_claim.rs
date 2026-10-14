use super::*;

use acme_kit::text;

/// Where a postmortem claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PostmortemClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PostmortemClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemClaim {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub margin: f64,
    pub grace_days: i64,
    pub region: String,
    pub quantity: i64,
    pub retries: i64,
    pub description: String,
    pub fingerprint: String,
    pub status: PostmortemClaimStatus,
}

impl PostmortemClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemClaimStatus) -> bool {
        match self.status {
            PostmortemClaimStatus::Proposed => {
                match next {
                    PostmortemClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemClaimStatus::Approved => {
                match next {
                    PostmortemClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemClaimStatus::Rejected => {
                return next == PostmortemClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem claim", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemClaim {
        PostmortemClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            margin: self.margin,
            grace_days: self.grace_days,
            region: self.region.clone(),
            quantity: self.quantity,
            retries: self.retries,
            description: self.description.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a postmortem claim in status proposed.
pub fn new_postmortem_claim(code: &str, label: &str) -> PostmortemClaim {
    PostmortemClaim {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        margin: 0.0,
        grace_days: 0,
        region: "".to_string(),
        quantity: 0,
        retries: 0,
        description: "".to_string(),
        fingerprint: "".to_string(),
        status: PostmortemClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_postmortem_claim("code-949", "label-988");
        assert_eq!(item.status, PostmortemClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_postmortem_claim("code-949", "label-988");
        if item.move_to(PostmortemClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_postmortem_claim("code-949", "label-988");
        item.status = PostmortemClaimStatus::Withdrawn;
        assert!(
            item.move_to(PostmortemClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
