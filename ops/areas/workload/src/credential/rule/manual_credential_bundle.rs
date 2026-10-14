use super::*;

use acme_kit::text;

/// Where a manual credential bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCredentialBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualCredentialBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualCredentialBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual credential bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundle {
    pub code: String,
    pub title: String,
    pub host: String,
    pub port: i64,
    pub duration_minutes: i64,
    pub status: ManualCredentialBundleStatus,
}

impl ManualCredentialBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCredentialBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCredentialBundleStatus) -> bool {
        match self.status {
            ManualCredentialBundleStatus::Proposed => {
                match next {
                    ManualCredentialBundleStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialBundleStatus::Approved => {
                match next {
                    ManualCredentialBundleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialBundleStatus::Rejected => {
                return next == ManualCredentialBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualCredentialBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual credential bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual credential bundle", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCredentialBundle {
        ManualCredentialBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            port: self.port,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual credential bundle in status proposed.
pub fn new_manual_credential_bundle(code: &str, title: &str) -> ManualCredentialBundle {
    ManualCredentialBundle {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        port: 0,
        duration_minutes: 0,
        status: ManualCredentialBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_credential_bundle("code-380", "title-47");
        assert_eq!(item.status, ManualCredentialBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_credential_bundle("code-380", "title-47");
        if item.move_to(ManualCredentialBundleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCredentialBundleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_credential_bundle("code-380", "title-47");
        item.status = ManualCredentialBundleStatus::Withdrawn;
        assert!(
            item.move_to(ManualCredentialBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
