use super::*;

use acme_kit::text;

/// Where a global freeze manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFreezeManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalFreezeManifestStatus {
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

impl std::fmt::Display for GlobalFreezeManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global freeze manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifest {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub channel: String,
    pub checksum: String,
    pub margin: f64,
    pub floor_value: f64,
    pub segment: String,
    pub principal: String,
    pub status: GlobalFreezeManifestStatus,
}

impl GlobalFreezeManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFreezeManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFreezeManifestStatus) -> bool {
        match self.status {
            GlobalFreezeManifestStatus::Planned => {
                return next == GlobalFreezeManifestStatus::InProgress || next == GlobalFreezeManifestStatus::Completed;
            }
            GlobalFreezeManifestStatus::Scheduled => {
                return next == GlobalFreezeManifestStatus::InProgress || next == GlobalFreezeManifestStatus::Cancelled;
            }
            GlobalFreezeManifestStatus::InProgress => {
                return next == GlobalFreezeManifestStatus::Cancelled;
            }
            GlobalFreezeManifestStatus::Completed => {
                return next == GlobalFreezeManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalFreezeManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global freeze manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global freeze manifest", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.floor_value - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFreezeManifest {
        GlobalFreezeManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            channel: self.channel.clone(),
            checksum: self.checksum.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global freeze manifest in status planned.
pub fn new_global_freeze_manifest(code: &str, title: &str) -> GlobalFreezeManifest {
    GlobalFreezeManifest {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        channel: "".to_string(),
        checksum: "".to_string(),
        margin: 0.0,
        floor_value: 0.0,
        segment: "".to_string(),
        principal: "".to_string(),
        status: GlobalFreezeManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_freeze_manifest("code-388", "title-915");
        assert_eq!(item.status, GlobalFreezeManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_freeze_manifest("code-388", "title-915");
        if item.move_to(GlobalFreezeManifestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFreezeManifestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_freeze_manifest("code-388", "title-915");
        item.status = GlobalFreezeManifestStatus::Cancelled;
        assert!(
            item.move_to(GlobalFreezeManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
