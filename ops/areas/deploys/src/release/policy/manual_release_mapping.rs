use super::*;

use acme_kit::text;

/// Where a manual release mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualReleaseMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualReleaseMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualReleaseMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual release mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMapping {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub name: String,
    pub batch_size: i64,
    pub namespace: String,
    pub margin: f64,
    pub status: ManualReleaseMappingStatus,
}

impl ManualReleaseMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualReleaseMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualReleaseMappingStatus) -> bool {
        match self.status {
            ManualReleaseMappingStatus::Proposed => {
                match next {
                    ManualReleaseMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualReleaseMappingStatus::Approved => {
                return next == ManualReleaseMappingStatus::Rejected || next == ManualReleaseMappingStatus::Withdrawn;
            }
            ManualReleaseMappingStatus::Rejected => {
                return next == ManualReleaseMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualReleaseMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual release mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual release mapping", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.margin * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualReleaseMapping {
        ManualReleaseMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            name: self.name.clone(),
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual release mapping in status proposed.
pub fn new_manual_release_mapping(code: &str, title: &str) -> ManualReleaseMapping {
    ManualReleaseMapping {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        name: "".to_string(),
        batch_size: 0,
        namespace: "".to_string(),
        margin: 0.0,
        status: ManualReleaseMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_release_mapping("code-626", "title-806");
        assert_eq!(item.status, ManualReleaseMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_release_mapping("code-626", "title-806");
        if item.move_to(ManualReleaseMappingStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualReleaseMappingStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_release_mapping("code-626", "title-806");
        item.status = ManualReleaseMappingStatus::Withdrawn;
        assert!(
            item.move_to(ManualReleaseMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
