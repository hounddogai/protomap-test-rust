use super::*;

use acme_kit::text;

/// Where a release mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReleaseMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReleaseMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMapping {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub currency: String,
    pub port: i64,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub cluster: String,
    pub status: ReleaseMappingStatus,
}

impl ReleaseMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseMappingStatus) -> bool {
        match self.status {
            ReleaseMappingStatus::Requested => {
                return next == ReleaseMappingStatus::Granted || next == ReleaseMappingStatus::Revoked;
            }
            ReleaseMappingStatus::Granted => {
                return next == ReleaseMappingStatus::Revoked || next == ReleaseMappingStatus::Expired;
            }
            ReleaseMappingStatus::Revoked => {
                return next == ReleaseMappingStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "release mapping", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.port as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseMapping {
        ReleaseMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            port: self.port,
            tags: self.tags.clone(),
            ratio: self.ratio,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a release mapping in status requested.
pub fn new_release_mapping(code: &str, name: &str) -> ReleaseMapping {
    ReleaseMapping {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        currency: "".to_string(),
        port: 0,
        tags: Vec::new(),
        ratio: 0.0,
        cluster: "".to_string(),
        status: ReleaseMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_release_mapping("code-627", "name-328");
        assert_eq!(item.status, ReleaseMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_release_mapping("code-627", "name-328");
        if item.move_to(ReleaseMappingStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseMappingStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_release_mapping("code-627", "name-328");
        item.status = ReleaseMappingStatus::Expired;
        assert!(
            item.move_to(ReleaseMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
