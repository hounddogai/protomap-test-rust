use super::*;

use acme_kit::text;

/// Where a identity bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl IdentityBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for IdentityBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity bundle as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBundle {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub channel: String,
    pub revision: i64,
    pub source: String,
    pub cost: f64,
    pub quota_units: i64,
    pub margin: f64,
    pub status: IdentityBundleStatus,
}

impl IdentityBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityBundleStatus) -> bool {
        match self.status {
            IdentityBundleStatus::Proposed => {
                return next == IdentityBundleStatus::Rejected;
            }
            IdentityBundleStatus::Approved => {
                return next == IdentityBundleStatus::Rejected || next == IdentityBundleStatus::Withdrawn;
            }
            IdentityBundleStatus::Rejected => {
                return next == IdentityBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity bundle {} cannot move from {} to {}",
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

    pub fn weighted_value(&self) -> f64 {
        ((self.revision as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityBundle {
        IdentityBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            channel: self.channel.clone(),
            revision: self.revision,
            source: self.source.clone(),
            cost: self.cost,
            quota_units: self.quota_units,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a identity bundle in status proposed.
pub fn new_identity_bundle(code: &str, label: &str) -> IdentityBundle {
    IdentityBundle {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        channel: "".to_string(),
        revision: 0,
        source: "".to_string(),
        cost: 0.0,
        quota_units: 0,
        margin: 0.0,
        status: IdentityBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_identity_bundle("code-149", "label-758");
        assert_eq!(item.status, IdentityBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_identity_bundle("code-149", "label-758");
        if item.move_to(IdentityBundleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityBundleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_identity_bundle("code-149", "label-758");
        item.status = IdentityBundleStatus::Withdrawn;
        assert!(
            item.move_to(IdentityBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
