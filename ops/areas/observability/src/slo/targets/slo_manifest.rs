use super::*;

use acme_kit::text;

/// Where a slo manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloManifestStatus {
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

impl std::fmt::Display for SloManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloManifest {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub region: String,
    pub minimum: f64,
    pub visible: bool,
    pub discount: f64,
    pub pinned: bool,
    pub tenant_id: String,
    pub status: SloManifestStatus,
}

impl SloManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloManifestStatus) -> bool {
        match self.status {
            SloManifestStatus::Planned => {
                match next {
                    SloManifestStatus::Scheduled | SloManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloManifestStatus::Scheduled => {
                match next {
                    SloManifestStatus::InProgress | SloManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloManifestStatus::InProgress => {
                return next == SloManifestStatus::Completed || next == SloManifestStatus::Cancelled;
            }
            SloManifestStatus::Completed => {
                match next {
                    SloManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SloManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.discount * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloManifest {
        SloManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            region: self.region.clone(),
            minimum: self.minimum,
            visible: self.visible,
            discount: self.discount,
            pinned: self.pinned,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo manifest in status planned.
pub fn new_slo_manifest(code: &str, title: &str) -> SloManifest {
    SloManifest {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        region: "".to_string(),
        minimum: 0.0,
        visible: false,
        discount: 0.0,
        pinned: false,
        tenant_id: "".to_string(),
        status: SloManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_manifest("code-530", "title-439");
        assert_eq!(item.status, SloManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_slo_manifest("code-530", "title-439");
        if item.move_to(SloManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_manifest("code-530", "title-439");
        item.status = SloManifestStatus::Cancelled;
        assert!(
            item.move_to(SloManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
