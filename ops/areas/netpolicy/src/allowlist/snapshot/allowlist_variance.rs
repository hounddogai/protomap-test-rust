use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a allowlist variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist variance as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVariance {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub starts_at: i64,
    pub status: AllowlistVarianceStatus,
}

impl AllowlistVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistVarianceStatus) -> bool {
        match self.status {
            AllowlistVarianceStatus::Proposed => {
                return next == AllowlistVarianceStatus::Rejected || next == AllowlistVarianceStatus::Withdrawn;
            }
            AllowlistVarianceStatus::Approved => {
                match next {
                    AllowlistVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistVarianceStatus::Rejected => {
                match next {
                    AllowlistVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AllowlistVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist variance", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> AllowlistVariance {
        AllowlistVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist variance in status proposed.
pub fn new_allowlist_variance(code: &str, name: &str) -> AllowlistVariance {
    AllowlistVariance {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        pinned: false,
        starts_at: 0,
        status: AllowlistVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_allowlist_variance("code-905", "name-957");
        assert_eq!(item.status, AllowlistVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_allowlist_variance("code-905", "name-957");
        if item.move_to(AllowlistVarianceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistVarianceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_allowlist_variance("code-905", "name-957");
        item.status = AllowlistVarianceStatus::Withdrawn;
        assert!(
            item.move_to(AllowlistVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
