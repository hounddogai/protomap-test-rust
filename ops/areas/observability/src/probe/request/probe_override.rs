use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a probe override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbeOverrideStatus {
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

impl std::fmt::Display for ProbeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverride {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub burst: i64,
    pub revision: i64,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub priority: i64,
    pub phone: String,
    pub duration_minutes: i64,
    pub category: String,
    pub status: ProbeOverrideStatus,
}

impl ProbeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeOverrideStatus) -> bool {
        match self.status {
            ProbeOverrideStatus::Open => {
                return next == ProbeOverrideStatus::Reopened;
            }
            ProbeOverrideStatus::InReview => {
                return next == ProbeOverrideStatus::Resolved || next == ProbeOverrideStatus::Reopened;
            }
            ProbeOverrideStatus::Resolved => {
                match next {
                    ProbeOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeOverrideStatus::Reopened => {
                match next {
                    ProbeOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: ProbeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeOverride {
        ProbeOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            burst: self.burst,
            revision: self.revision,
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            priority: self.priority,
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a probe override in status open.
pub fn new_probe_override(code: &str, name: &str) -> ProbeOverride {
    ProbeOverride {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        burst: 0,
        revision: 0,
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        priority: 0,
        phone: "".to_string(),
        duration_minutes: 0,
        category: "".to_string(),
        status: ProbeOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_override("code-466", "name-130");
        assert_eq!(item.status, ProbeOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_probe_override("code-466", "name-130");
        if item.move_to(ProbeOverrideStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeOverrideStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_override("code-466", "name-130");
        item.status = ProbeOverrideStatus::Archived;
        assert!(
            item.move_to(ProbeOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
