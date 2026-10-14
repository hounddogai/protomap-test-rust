use super::*;

use acme_kit::text;

/// Where a policy profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PolicyProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PolicyProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfile {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub expires_at: i64,
    pub phone: String,
    pub batch_size: i64,
    pub external_ref: String,
    pub issuer: String,
    pub country: String,
    pub quota_units: i64,
    pub status: PolicyProfileStatus,
}

impl PolicyProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyProfileStatus) -> bool {
        match self.status {
            PolicyProfileStatus::Proposed => {
                match next {
                    PolicyProfileStatus::Approved | PolicyProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyProfileStatus::Approved => {
                match next {
                    PolicyProfileStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyProfileStatus::Rejected => {
                match next {
                    PolicyProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PolicyProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy profile {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyProfile {
        PolicyProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a policy profile in status proposed.
pub fn new_policy_profile(code: &str, label: &str) -> PolicyProfile {
    PolicyProfile {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        expires_at: 0,
        phone: "".to_string(),
        batch_size: 0,
        external_ref: "".to_string(),
        issuer: "".to_string(),
        country: "".to_string(),
        quota_units: 0,
        status: PolicyProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_policy_profile("code-749", "label-496");
        assert_eq!(item.status, PolicyProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_policy_profile("code-749", "label-496");
        if item.move_to(PolicyProfileStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyProfileStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_policy_profile("code-749", "label-496");
        item.status = PolicyProfileStatus::Withdrawn;
        assert!(
            item.move_to(PolicyProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
