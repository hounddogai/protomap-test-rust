use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rollout manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RolloutManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifest {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub contact_email: String,
    pub active: bool,
    pub burst: i64,
    pub threshold: f64,
    pub status: RolloutManifestStatus,
}

impl RolloutManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutManifestStatus) -> bool {
        match self.status {
            RolloutManifestStatus::New => {
                match next {
                    RolloutManifestStatus::Queued | RolloutManifestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutManifestStatus::Queued => {
                return next == RolloutManifestStatus::Running || next == RolloutManifestStatus::Done;
            }
            RolloutManifestStatus::Running => {
                match next {
                    RolloutManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutManifestStatus::Done => {
                return next == RolloutManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.burst as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutManifest {
        RolloutManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            burst: self.burst,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout manifest in status new.
pub fn new_rollout_manifest(code: &str, name: &str) -> RolloutManifest {
    RolloutManifest {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        capacity: 0,
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        active: false,
        burst: 0,
        threshold: 0.0,
        status: RolloutManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_manifest("code-858", "name-298");
        assert_eq!(item.status, RolloutManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollout_manifest("code-858", "name-298");
        if item.move_to(RolloutManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_manifest("code-858", "name-298");
        item.status = RolloutManifestStatus::Failed;
        assert!(
            item.move_to(RolloutManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
