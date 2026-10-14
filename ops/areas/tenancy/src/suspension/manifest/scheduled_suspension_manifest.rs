use super::*;

use acme_kit::text;

/// Where a scheduled suspension manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledSuspensionManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledSuspensionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled suspension manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifest {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub score: f64,
    pub name: String,
    pub segment: String,
    pub summary: String,
    pub fingerprint: String,
    pub archived: bool,
    pub tenant_id: String,
    pub limit: i64,
    pub status: ScheduledSuspensionManifestStatus,
}

impl ScheduledSuspensionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionManifestStatus) -> bool {
        match self.status {
            ScheduledSuspensionManifestStatus::Idle => {
                return next == ScheduledSuspensionManifestStatus::Draining;
            }
            ScheduledSuspensionManifestStatus::Warming => {
                return next == ScheduledSuspensionManifestStatus::Draining
                    || next == ScheduledSuspensionManifestStatus::Stopped;
            }
            ScheduledSuspensionManifestStatus::Ready => {
                return next == ScheduledSuspensionManifestStatus::Draining
                    || next == ScheduledSuspensionManifestStatus::Stopped;
            }
            ScheduledSuspensionManifestStatus::Draining => {
                match next {
                    ScheduledSuspensionManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledSuspensionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionManifest {
        ScheduledSuspensionManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            score: self.score,
            name: self.name.clone(),
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled suspension manifest in status idle.
pub fn new_scheduled_suspension_manifest(code: &str, label: &str) -> ScheduledSuspensionManifest {
    ScheduledSuspensionManifest {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        score: 0.0,
        name: "".to_string(),
        segment: "".to_string(),
        summary: "".to_string(),
        fingerprint: "".to_string(),
        archived: false,
        tenant_id: "".to_string(),
        limit: 0,
        status: ScheduledSuspensionManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_suspension_manifest("code-973", "label-91");
        assert_eq!(item.status, ScheduledSuspensionManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_scheduled_suspension_manifest("code-973", "label-91");
        if item.move_to(ScheduledSuspensionManifestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSuspensionManifestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_suspension_manifest("code-973", "label-91");
        item.status = ScheduledSuspensionManifestStatus::Stopped;
        assert!(
            item.move_to(ScheduledSuspensionManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
