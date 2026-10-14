use super::*;

use acme_kit::text;

/// Where a manual retention bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRetentionBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualRetentionBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ManualRetentionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual retention bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundle {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub region: String,
    pub approved: bool,
    pub attempts: i64,
    pub revision: i64,
    pub country: String,
    pub source: String,
    pub port: i64,
    pub ceiling: f64,
    pub status: ManualRetentionBundleStatus,
}

impl ManualRetentionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRetentionBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRetentionBundleStatus) -> bool {
        match self.status {
            ManualRetentionBundleStatus::Draft => {
                match next {
                    ManualRetentionBundleStatus::Pending | ManualRetentionBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRetentionBundleStatus::Pending => {
                return next == ManualRetentionBundleStatus::Suspended || next == ManualRetentionBundleStatus::Closed;
            }
            ManualRetentionBundleStatus::Active => {
                match next {
                    ManualRetentionBundleStatus::Suspended | ManualRetentionBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRetentionBundleStatus::Suspended => {
                match next {
                    ManualRetentionBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualRetentionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual retention bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quota_units as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRetentionBundle {
        ManualRetentionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            approved: self.approved,
            attempts: self.attempts,
            revision: self.revision,
            country: self.country.clone(),
            source: self.source.clone(),
            port: self.port,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual retention bundle in status draft.
pub fn new_manual_retention_bundle(code: &str, label: &str) -> ManualRetentionBundle {
    ManualRetentionBundle {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        region: "".to_string(),
        approved: false,
        attempts: 0,
        revision: 0,
        country: "".to_string(),
        source: "".to_string(),
        port: 0,
        ceiling: 0.0,
        status: ManualRetentionBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_retention_bundle("code-733", "label-639");
        assert_eq!(item.status, ManualRetentionBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_manual_retention_bundle("code-733", "label-639");
        if item.move_to(ManualRetentionBundleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRetentionBundleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_retention_bundle("code-733", "label-639");
        item.status = ManualRetentionBundleStatus::Closed;
        assert!(
            item.move_to(ManualRetentionBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
