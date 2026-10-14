use super::*;

use acme_kit::text;

/// Where a regional evidence manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEvidenceManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalEvidenceManifestStatus {
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

impl std::fmt::Display for RegionalEvidenceManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional evidence manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifest {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub version: i64,
    pub locale: String,
    pub duration_minutes: i64,
    pub status: RegionalEvidenceManifestStatus,
}

impl RegionalEvidenceManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEvidenceManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEvidenceManifestStatus) -> bool {
        match self.status {
            RegionalEvidenceManifestStatus::Idle => {
                match next {
                    RegionalEvidenceManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEvidenceManifestStatus::Warming => {
                return next == RegionalEvidenceManifestStatus::Ready
                    || next == RegionalEvidenceManifestStatus::Draining;
            }
            RegionalEvidenceManifestStatus::Ready => {
                match next {
                    RegionalEvidenceManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEvidenceManifestStatus::Draining => {
                return next == RegionalEvidenceManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalEvidenceManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional evidence manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.version as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalEvidenceManifest {
        RegionalEvidenceManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            version: self.version,
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional evidence manifest in status idle.
pub fn new_regional_evidence_manifest(code: &str, title: &str) -> RegionalEvidenceManifest {
    RegionalEvidenceManifest {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        version: 0,
        locale: "".to_string(),
        duration_minutes: 0,
        status: RegionalEvidenceManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_evidence_manifest("code-520", "title-925");
        assert_eq!(item.status, RegionalEvidenceManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_regional_evidence_manifest("code-520", "title-925");
        if item.move_to(RegionalEvidenceManifestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalEvidenceManifestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_evidence_manifest("code-520", "title-925");
        item.status = RegionalEvidenceManifestStatus::Stopped;
        assert!(
            item.move_to(RegionalEvidenceManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
