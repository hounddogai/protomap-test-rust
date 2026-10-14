use super::*;

use acme_kit::text;

/// Where a policy group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PolicyGroupStatus {
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

impl std::fmt::Display for PolicyGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGroup {
    pub code: String,
    pub title: String,
    pub region: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub description: String,
    pub rotation_days: i64,
    pub status: PolicyGroupStatus,
}

impl PolicyGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyGroupStatus) -> bool {
        match self.status {
            PolicyGroupStatus::Open => {
                match next {
                    PolicyGroupStatus::Resolved | PolicyGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyGroupStatus::InReview => {
                match next {
                    PolicyGroupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyGroupStatus::Resolved => {
                match next {
                    PolicyGroupStatus::Reopened | PolicyGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyGroupStatus::Reopened => {
                match next {
                    PolicyGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: PolicyGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy group", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.rotation_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyGroup {
        PolicyGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a policy group in status open.
pub fn new_policy_group(code: &str, title: &str) -> PolicyGroup {
    PolicyGroup {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        description: "".to_string(),
        rotation_days: 0,
        status: PolicyGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_policy_group("code-223", "title-288");
        assert_eq!(item.status, PolicyGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_policy_group("code-223", "title-288");
        if item.move_to(PolicyGroupStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyGroupStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_policy_group("code-223", "title-288");
        item.status = PolicyGroupStatus::Archived;
        assert!(item.move_to(PolicyGroupStatus::Open).is_err(), "expected move to fail");
    }
}
