use super::*;

use acme_kit::text;

/// Where a usage request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage request as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequest {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub issuer: String,
    pub quantity: i64,
    pub weight: f64,
    pub name: String,
    pub discount: f64,
    pub revision: i64,
    pub approved: bool,
    pub status: UsageRequestStatus,
}

impl UsageRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageRequestStatus) -> bool {
        match self.status {
            UsageRequestStatus::Proposed => {
                return next == UsageRequestStatus::Withdrawn;
            }
            UsageRequestStatus::Approved => {
                return next == UsageRequestStatus::Rejected;
            }
            UsageRequestStatus::Rejected => {
                return next == UsageRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "usage request", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.weight + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageRequest {
        UsageRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            weight: self.weight,
            name: self.name.clone(),
            discount: self.discount,
            revision: self.revision,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage request in status proposed.
pub fn new_usage_request(code: &str, label: &str) -> UsageRequest {
    UsageRequest {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        issuer: "".to_string(),
        quantity: 0,
        weight: 0.0,
        name: "".to_string(),
        discount: 0.0,
        revision: 0,
        approved: false,
        status: UsageRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_request("code-335", "label-64");
        assert_eq!(item.status, UsageRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_usage_request("code-335", "label-64");
        if item.move_to(UsageRequestStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageRequestStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_request("code-335", "label-64");
        item.status = UsageRequestStatus::Withdrawn;
        assert!(
            item.move_to(UsageRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
