use super::*;

use acme_kit::text;

/// Where a approval mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMapping {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub attempts: i64,
    pub rate: f64,
    pub window_days: i64,
    pub cluster: String,
    pub segment: String,
    pub quantity: i64,
    pub category: String,
    pub active: bool,
    pub status: ApprovalMappingStatus,
}

impl ApprovalMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalMappingStatus) -> bool {
        match self.status {
            ApprovalMappingStatus::Created => {
                return next == ApprovalMappingStatus::Validated;
            }
            ApprovalMappingStatus::Validated => {
                return next == ApprovalMappingStatus::Published;
            }
            ApprovalMappingStatus::Published => {
                match next {
                    ApprovalMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: ApprovalMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.attempts as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalMapping {
        ApprovalMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            rate: self.rate,
            window_days: self.window_days,
            cluster: self.cluster.clone(),
            segment: self.segment.clone(),
            quantity: self.quantity,
            category: self.category.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a approval mapping in status created.
pub fn new_approval_mapping(code: &str, label: &str) -> ApprovalMapping {
    ApprovalMapping {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        attempts: 0,
        rate: 0.0,
        window_days: 0,
        cluster: "".to_string(),
        segment: "".to_string(),
        quantity: 0,
        category: "".to_string(),
        active: false,
        status: ApprovalMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_mapping("code-39", "label-786");
        assert_eq!(item.status, ApprovalMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_approval_mapping("code-39", "label-786");
        if item.move_to(ApprovalMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_mapping("code-39", "label-786");
        item.status = ApprovalMappingStatus::Retired;
        assert!(
            item.move_to(ApprovalMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
