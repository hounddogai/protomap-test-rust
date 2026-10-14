use super::*;

use acme_kit::text;

/// Where a default egress batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEgressBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultEgressBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultEgressBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default egress batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatch {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub discount: f64,
    pub visible: bool,
    pub region: String,
    pub archived: bool,
    pub name: String,
    pub quantity: i64,
    pub status: DefaultEgressBatchStatus,
}

impl DefaultEgressBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEgressBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEgressBatchStatus) -> bool {
        match self.status {
            DefaultEgressBatchStatus::Requested => {
                return next == DefaultEgressBatchStatus::Revoked || next == DefaultEgressBatchStatus::Expired;
            }
            DefaultEgressBatchStatus::Granted => {
                return next == DefaultEgressBatchStatus::Revoked || next == DefaultEgressBatchStatus::Expired;
            }
            DefaultEgressBatchStatus::Revoked => {
                return next == DefaultEgressBatchStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEgressBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default egress batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEgressBatch {
        DefaultEgressBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            discount: self.discount,
            visible: self.visible,
            region: self.region.clone(),
            archived: self.archived,
            name: self.name.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default egress batch in status requested.
pub fn new_default_egress_batch(code: &str, label: &str) -> DefaultEgressBatch {
    DefaultEgressBatch {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        discount: 0.0,
        visible: false,
        region: "".to_string(),
        archived: false,
        name: "".to_string(),
        quantity: 0,
        status: DefaultEgressBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_egress_batch("code-35", "label-626");
        assert_eq!(item.status, DefaultEgressBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_egress_batch("code-35", "label-626");
        if item.move_to(DefaultEgressBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEgressBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_egress_batch("code-35", "label-626");
        item.status = DefaultEgressBatchStatus::Expired;
        assert!(
            item.move_to(DefaultEgressBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
