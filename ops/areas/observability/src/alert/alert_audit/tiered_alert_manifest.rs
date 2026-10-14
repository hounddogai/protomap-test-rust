use super::*;

use acme_kit::text;

/// Where a tiered alert manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredAlertManifestStatus {
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

impl std::fmt::Display for TieredAlertManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered alert manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertManifest {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub capacity: i64,
    pub archived: bool,
    pub scope: String,
    pub status: TieredAlertManifestStatus,
}

impl TieredAlertManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertManifestStatus) -> bool {
        match self.status {
            TieredAlertManifestStatus::New => {
                return next == TieredAlertManifestStatus::Queued || next == TieredAlertManifestStatus::Done;
            }
            TieredAlertManifestStatus::Queued => {
                return next == TieredAlertManifestStatus::Done;
            }
            TieredAlertManifestStatus::Running => {
                match next {
                    TieredAlertManifestStatus::Done | TieredAlertManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertManifestStatus::Done => {
                return next == TieredAlertManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredAlertManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered alert manifest", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.capacity as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAlertManifest {
        TieredAlertManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            capacity: self.capacity,
            archived: self.archived,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered alert manifest in status new.
pub fn new_tiered_alert_manifest(code: &str, label: &str) -> TieredAlertManifest {
    TieredAlertManifest {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        capacity: 0,
        archived: false,
        scope: "".to_string(),
        status: TieredAlertManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_alert_manifest("code-618", "label-143");
        assert_eq!(item.status, TieredAlertManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_alert_manifest("code-618", "label-143");
        if item.move_to(TieredAlertManifestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAlertManifestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_alert_manifest("code-618", "label-143");
        item.status = TieredAlertManifestStatus::Failed;
        assert!(
            item.move_to(TieredAlertManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
