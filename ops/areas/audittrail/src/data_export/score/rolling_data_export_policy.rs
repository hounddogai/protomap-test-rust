use super::*;

use acme_kit::text;

/// Where a rolling data export policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDataExportPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingDataExportPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingDataExportPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling data export policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportPolicy {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub priority: i64,
    pub retries: i64,
    pub discount: f64,
    pub minimum: f64,
    pub scope: String,
    pub namespace: String,
    pub status: RollingDataExportPolicyStatus,
}

impl RollingDataExportPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDataExportPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDataExportPolicyStatus) -> bool {
        match self.status {
            RollingDataExportPolicyStatus::Proposed => {
                match next {
                    RollingDataExportPolicyStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportPolicyStatus::Approved => {
                match next {
                    RollingDataExportPolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportPolicyStatus::Rejected => {
                return next == RollingDataExportPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDataExportPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling data export policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.discount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDataExportPolicy {
        RollingDataExportPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            priority: self.priority,
            retries: self.retries,
            discount: self.discount,
            minimum: self.minimum,
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rolling data export policy in status proposed.
pub fn new_rolling_data_export_policy(code: &str, label: &str) -> RollingDataExportPolicy {
    RollingDataExportPolicy {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        priority: 0,
        retries: 0,
        discount: 0.0,
        minimum: 0.0,
        scope: "".to_string(),
        namespace: "".to_string(),
        status: RollingDataExportPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_data_export_policy("code-556", "label-823");
        assert_eq!(item.status, RollingDataExportPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_data_export_policy("code-556", "label-823");
        if item.move_to(RollingDataExportPolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDataExportPolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_data_export_policy("code-556", "label-823");
        item.status = RollingDataExportPolicyStatus::Withdrawn;
        assert!(
            item.move_to(RollingDataExportPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
