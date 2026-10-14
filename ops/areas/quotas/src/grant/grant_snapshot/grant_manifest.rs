use super::*;

use acme_kit::text;

/// Where a grant manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantManifestStatus {
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

impl std::fmt::Display for GrantManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifest {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub quantity: i64,
    pub revision: i64,
    pub external_ref: String,
    pub scope: String,
    pub burst: i64,
    pub status: GrantManifestStatus,
}

impl GrantManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantManifestStatus) -> bool {
        match self.status {
            GrantManifestStatus::New => {
                match next {
                    GrantManifestStatus::Queued | GrantManifestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantManifestStatus::Queued => {
                match next {
                    GrantManifestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantManifestStatus::Running => {
                return next == GrantManifestStatus::Done;
            }
            GrantManifestStatus::Done => {
                return next == GrantManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantManifest {
        GrantManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            quantity: self.quantity,
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a grant manifest in status new.
pub fn new_grant_manifest(code: &str, title: &str) -> GrantManifest {
    GrantManifest {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        quantity: 0,
        revision: 0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        burst: 0,
        status: GrantManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_manifest("code-526", "title-707");
        assert_eq!(item.status, GrantManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_grant_manifest("code-526", "title-707");
        if item.move_to(GrantManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_manifest("code-526", "title-707");
        item.status = GrantManifestStatus::Failed;
        assert!(item.move_to(GrantManifestStatus::New).is_err(), "expected move to fail");
    }
}
