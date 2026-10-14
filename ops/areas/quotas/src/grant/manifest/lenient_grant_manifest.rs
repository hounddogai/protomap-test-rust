use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient grant manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientGrantManifestStatus {
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

impl std::fmt::Display for LenientGrantManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient grant manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantManifest {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub version: i64,
    pub weight: f64,
    pub namespace: String,
    pub limit: i64,
    pub ceiling: f64,
    pub status: LenientGrantManifestStatus,
}

impl LenientGrantManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantManifestStatus) -> bool {
        match self.status {
            LenientGrantManifestStatus::Planned => {
                match next {
                    LenientGrantManifestStatus::Completed | LenientGrantManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientGrantManifestStatus::Scheduled => {
                match next {
                    LenientGrantManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientGrantManifestStatus::InProgress => {
                return next == LenientGrantManifestStatus::Completed || next == LenientGrantManifestStatus::Cancelled;
            }
            LenientGrantManifestStatus::Completed => {
                match next {
                    LenientGrantManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientGrantManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn projected_total(&self) -> f64 {
        (self.ceiling * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantManifest {
        LenientGrantManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            version: self.version,
            weight: self.weight,
            namespace: self.namespace.clone(),
            limit: self.limit,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient grant manifest in status planned.
pub fn new_lenient_grant_manifest(code: &str, name: &str) -> LenientGrantManifest {
    LenientGrantManifest {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        version: 0,
        weight: 0.0,
        namespace: "".to_string(),
        limit: 0,
        ceiling: 0.0,
        status: LenientGrantManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_grant_manifest("code-201", "name-55");
        assert_eq!(item.status, LenientGrantManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_grant_manifest("code-201", "name-55");
        if item.move_to(LenientGrantManifestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientGrantManifestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_grant_manifest("code-201", "name-55");
        item.status = LenientGrantManifestStatus::Cancelled;
        assert!(
            item.move_to(LenientGrantManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
