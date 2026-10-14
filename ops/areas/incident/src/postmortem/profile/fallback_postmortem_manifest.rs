use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback postmortem manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackPostmortemManifestStatus {
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

impl std::fmt::Display for FallbackPostmortemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback postmortem manifest as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemManifest {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub priority: i64,
    pub label: String,
    pub tags: Vec<String>,
    pub namespace: String,
    pub status: FallbackPostmortemManifestStatus,
}

impl FallbackPostmortemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemManifestStatus) -> bool {
        match self.status {
            FallbackPostmortemManifestStatus::New => {
                return next == FallbackPostmortemManifestStatus::Queued;
            }
            FallbackPostmortemManifestStatus::Queued => {
                match next {
                    FallbackPostmortemManifestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemManifestStatus::Running => {
                match next {
                    FallbackPostmortemManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemManifestStatus::Done => {
                return next == FallbackPostmortemManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPostmortemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback postmortem manifest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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
        ((self.priority as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemManifest {
        FallbackPostmortemManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            priority: self.priority,
            label: self.label.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback postmortem manifest in status new.
pub fn new_fallback_postmortem_manifest(code: &str, name: &str) -> FallbackPostmortemManifest {
    FallbackPostmortemManifest {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        country: "".to_string(),
        attributes: BTreeMap::new(),
        priority: 0,
        label: "".to_string(),
        tags: Vec::new(),
        namespace: "".to_string(),
        status: FallbackPostmortemManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_postmortem_manifest("code-477", "name-780");
        assert_eq!(item.status, FallbackPostmortemManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_postmortem_manifest("code-477", "name-780");
        if item.move_to(FallbackPostmortemManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_postmortem_manifest("code-477", "name-780");
        item.status = FallbackPostmortemManifestStatus::Failed;
        assert!(
            item.move_to(FallbackPostmortemManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
