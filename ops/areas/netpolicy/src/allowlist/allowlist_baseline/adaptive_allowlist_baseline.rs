use super::*;

use acme_kit::text;

/// Where a adaptive allowlist baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAllowlistBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveAllowlistBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveAllowlistBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive allowlist baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBaseline {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub priority: i64,
    pub retries: i64,
    pub discount: f64,
    pub country: String,
    pub version: i64,
    pub description: String,
    pub status: AdaptiveAllowlistBaselineStatus,
}

impl AdaptiveAllowlistBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAllowlistBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAllowlistBaselineStatus) -> bool {
        match self.status {
            AdaptiveAllowlistBaselineStatus::Requested => {
                match next {
                    AdaptiveAllowlistBaselineStatus::Granted | AdaptiveAllowlistBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBaselineStatus::Granted => {
                match next {
                    AdaptiveAllowlistBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBaselineStatus::Revoked => {
                return next == AdaptiveAllowlistBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveAllowlistBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive allowlist baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive allowlist baseline", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAllowlistBaseline {
        AdaptiveAllowlistBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            priority: self.priority,
            retries: self.retries,
            discount: self.discount,
            country: self.country.clone(),
            version: self.version,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive allowlist baseline in status requested.
pub fn new_adaptive_allowlist_baseline(code: &str, title: &str) -> AdaptiveAllowlistBaseline {
    AdaptiveAllowlistBaseline {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        priority: 0,
        retries: 0,
        discount: 0.0,
        country: "".to_string(),
        version: 0,
        description: "".to_string(),
        status: AdaptiveAllowlistBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_allowlist_baseline("code-169", "title-849");
        assert_eq!(item.status, AdaptiveAllowlistBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_adaptive_allowlist_baseline("code-169", "title-849");
        if item.move_to(AdaptiveAllowlistBaselineStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAllowlistBaselineStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_allowlist_baseline("code-169", "title-849");
        item.status = AdaptiveAllowlistBaselineStatus::Expired;
        assert!(
            item.move_to(AdaptiveAllowlistBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
