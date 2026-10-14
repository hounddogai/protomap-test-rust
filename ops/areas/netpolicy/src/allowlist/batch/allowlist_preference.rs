use super::*;

use acme_kit::text;

/// Where a allowlist preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistPreference {
    pub code: String,
    pub name: String,
    pub title: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub ratio: f64,
    pub issuer: String,
    pub timezone: String,
    pub owner_id: String,
    pub retries: i64,
    pub approved: bool,
    pub status: AllowlistPreferenceStatus,
}

impl AllowlistPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistPreferenceStatus) -> bool {
        match self.status {
            AllowlistPreferenceStatus::Requested => {
                match next {
                    AllowlistPreferenceStatus::Granted | AllowlistPreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistPreferenceStatus::Granted => {
                return next == AllowlistPreferenceStatus::Revoked;
            }
            AllowlistPreferenceStatus::Revoked => {
                match next {
                    AllowlistPreferenceStatus::Expired => {
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

    pub fn move_to(&mut self, next: AllowlistPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ratio + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistPreference {
        AllowlistPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            score: self.score,
            ratio: self.ratio,
            issuer: self.issuer.clone(),
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            retries: self.retries,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist preference in status requested.
pub fn new_allowlist_preference(code: &str, name: &str) -> AllowlistPreference {
    AllowlistPreference {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        tags: Vec::new(),
        score: 0.0,
        ratio: 0.0,
        issuer: "".to_string(),
        timezone: "".to_string(),
        owner_id: "".to_string(),
        retries: 0,
        approved: false,
        status: AllowlistPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_preference("code-874", "name-959");
        assert_eq!(item.status, AllowlistPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_allowlist_preference("code-874", "name-959");
        if item.move_to(AllowlistPreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistPreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_preference("code-874", "name-959");
        item.status = AllowlistPreferenceStatus::Expired;
        assert!(
            item.move_to(AllowlistPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
