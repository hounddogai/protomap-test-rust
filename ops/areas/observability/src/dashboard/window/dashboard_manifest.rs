use super::*;

use acme_kit::text;

/// Where a dashboard manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DashboardManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DashboardManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard manifest as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifest {
    pub code: String,
    pub label: String,
    pub region: String,
    pub namespace: String,
    pub name: String,
    pub attempts: i64,
    pub capacity: i64,
    pub status: DashboardManifestStatus,
}

impl DashboardManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardManifestStatus) -> bool {
        match self.status {
            DashboardManifestStatus::Requested => {
                return next == DashboardManifestStatus::Revoked || next == DashboardManifestStatus::Expired;
            }
            DashboardManifestStatus::Granted => {
                match next {
                    DashboardManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardManifestStatus::Revoked => {
                match next {
                    DashboardManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: DashboardManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardManifest {
        DashboardManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a dashboard manifest in status requested.
pub fn new_dashboard_manifest(code: &str, label: &str) -> DashboardManifest {
    DashboardManifest {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        namespace: "".to_string(),
        name: "".to_string(),
        attempts: 0,
        capacity: 0,
        status: DashboardManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dashboard_manifest("code-503", "label-624");
        assert_eq!(item.status, DashboardManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_dashboard_manifest("code-503", "label-624");
        if item.move_to(DashboardManifestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardManifestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dashboard_manifest("code-503", "label-624");
        item.status = DashboardManifestStatus::Expired;
        assert!(
            item.move_to(DashboardManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
