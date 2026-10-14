use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient redaction manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRedactionManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientRedactionManifestStatus {
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

impl std::fmt::Display for LenientRedactionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient redaction manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifest {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub principal: String,
    pub version: i64,
    pub threshold: f64,
    pub description: String,
    pub score: f64,
    pub status: LenientRedactionManifestStatus,
}

impl LenientRedactionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRedactionManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRedactionManifestStatus) -> bool {
        match self.status {
            LenientRedactionManifestStatus::New => {
                return next == LenientRedactionManifestStatus::Queued
                    || next == LenientRedactionManifestStatus::Failed;
            }
            LenientRedactionManifestStatus::Queued => {
                return next == LenientRedactionManifestStatus::Running
                    || next == LenientRedactionManifestStatus::Failed;
            }
            LenientRedactionManifestStatus::Running => {
                match next {
                    LenientRedactionManifestStatus::Done | LenientRedactionManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionManifestStatus::Done => {
                return next == LenientRedactionManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRedactionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient redaction manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRedactionManifest {
        LenientRedactionManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            principal: self.principal.clone(),
            version: self.version,
            threshold: self.threshold,
            description: self.description.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient redaction manifest in status new.
pub fn new_lenient_redaction_manifest(code: &str, title: &str) -> LenientRedactionManifest {
    LenientRedactionManifest {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        principal: "".to_string(),
        version: 0,
        threshold: 0.0,
        description: "".to_string(),
        score: 0.0,
        status: LenientRedactionManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_redaction_manifest("code-775", "title-82");
        assert_eq!(item.status, LenientRedactionManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_redaction_manifest("code-775", "title-82");
        if item.move_to(LenientRedactionManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRedactionManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_redaction_manifest("code-775", "title-82");
        item.status = LenientRedactionManifestStatus::Failed;
        assert!(
            item.move_to(LenientRedactionManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
