use super::*;

use acme_kit::text;

/// Where a fallback overage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackOverageManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackOverageManifestStatus {
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

impl std::fmt::Display for FallbackOverageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback overage manifest as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageManifest {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub limit: i64,
    pub capacity: i64,
    pub status: FallbackOverageManifestStatus,
}

impl FallbackOverageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackOverageManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackOverageManifestStatus) -> bool {
        match self.status {
            FallbackOverageManifestStatus::New => {
                match next {
                    FallbackOverageManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageManifestStatus::Queued => {
                match next {
                    FallbackOverageManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageManifestStatus::Running => {
                match next {
                    FallbackOverageManifestStatus::Done | FallbackOverageManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageManifestStatus::Done => {
                match next {
                    FallbackOverageManifestStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackOverageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback overage manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        ((self.limit as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackOverageManifest {
        FallbackOverageManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            limit: self.limit,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback overage manifest in status new.
pub fn new_fallback_overage_manifest(code: &str, label: &str) -> FallbackOverageManifest {
    FallbackOverageManifest {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        limit: 0,
        capacity: 0,
        status: FallbackOverageManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_overage_manifest("code-670", "label-974");
        assert_eq!(item.status, FallbackOverageManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_fallback_overage_manifest("code-670", "label-974");
        if item.move_to(FallbackOverageManifestStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackOverageManifestStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_overage_manifest("code-670", "label-974");
        item.status = FallbackOverageManifestStatus::Failed;
        assert!(
            item.move_to(FallbackOverageManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
