use super::*;

use acme_kit::text;

/// Where a fallback slo manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSloManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackSloManifestStatus {
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

impl std::fmt::Display for FallbackSloManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback slo manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifest {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub title: String,
    pub namespace: String,
    pub discount: f64,
    pub approved: bool,
    pub created_at: i64,
    pub status: FallbackSloManifestStatus,
}

impl FallbackSloManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSloManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSloManifestStatus) -> bool {
        match self.status {
            FallbackSloManifestStatus::New => {
                return next == FallbackSloManifestStatus::Queued || next == FallbackSloManifestStatus::Running;
            }
            FallbackSloManifestStatus::Queued => {
                return next == FallbackSloManifestStatus::Done;
            }
            FallbackSloManifestStatus::Running => {
                match next {
                    FallbackSloManifestStatus::Done | FallbackSloManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSloManifestStatus::Done => {
                return next == FallbackSloManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSloManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback slo manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        (self.discount * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSloManifest {
        FallbackSloManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            approved: self.approved,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback slo manifest in status new.
pub fn new_fallback_slo_manifest(code: &str, name: &str) -> FallbackSloManifest {
    FallbackSloManifest {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        title: "".to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        approved: false,
        created_at: 0,
        status: FallbackSloManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_slo_manifest("code-215", "name-254");
        assert_eq!(item.status, FallbackSloManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_slo_manifest("code-215", "name-254");
        if item.move_to(FallbackSloManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSloManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_slo_manifest("code-215", "name-254");
        item.status = FallbackSloManifestStatus::Failed;
        assert!(
            item.move_to(FallbackSloManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
