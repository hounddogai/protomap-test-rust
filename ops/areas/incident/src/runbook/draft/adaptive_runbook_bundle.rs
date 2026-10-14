use super::*;

use acme_kit::text;

/// Where a adaptive runbook bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRunbookBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveRunbookBundleStatus {
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

impl std::fmt::Display for AdaptiveRunbookBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive runbook bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundle {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub summary: String,
    pub timezone: String,
    pub version: i64,
    pub owner_id: String,
    pub locale: String,
    pub key_id: String,
    pub scope: String,
    pub status: AdaptiveRunbookBundleStatus,
}

impl AdaptiveRunbookBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRunbookBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRunbookBundleStatus) -> bool {
        match self.status {
            AdaptiveRunbookBundleStatus::New => {
                return next == AdaptiveRunbookBundleStatus::Failed;
            }
            AdaptiveRunbookBundleStatus::Queued => {
                return next == AdaptiveRunbookBundleStatus::Running || next == AdaptiveRunbookBundleStatus::Done;
            }
            AdaptiveRunbookBundleStatus::Running => {
                return next == AdaptiveRunbookBundleStatus::Done || next == AdaptiveRunbookBundleStatus::Failed;
            }
            AdaptiveRunbookBundleStatus::Done => {
                match next {
                    AdaptiveRunbookBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveRunbookBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive runbook bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> AdaptiveRunbookBundle {
        AdaptiveRunbookBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            version: self.version,
            owner_id: self.owner_id.clone(),
            locale: self.locale.clone(),
            key_id: self.key_id.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive runbook bundle in status new.
pub fn new_adaptive_runbook_bundle(code: &str, name: &str) -> AdaptiveRunbookBundle {
    AdaptiveRunbookBundle {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        summary: "".to_string(),
        timezone: "".to_string(),
        version: 0,
        owner_id: "".to_string(),
        locale: "".to_string(),
        key_id: "".to_string(),
        scope: "".to_string(),
        status: AdaptiveRunbookBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_runbook_bundle("code-668", "name-679");
        assert_eq!(item.status, AdaptiveRunbookBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_adaptive_runbook_bundle("code-668", "name-679");
        if item.move_to(AdaptiveRunbookBundleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRunbookBundleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_runbook_bundle("code-668", "name-679");
        item.status = AdaptiveRunbookBundleStatus::Failed;
        assert!(
            item.move_to(AdaptiveRunbookBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
