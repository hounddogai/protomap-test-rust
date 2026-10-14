use super::*;

use acme_kit::text;

/// Where a approval threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThreshold {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub window_days: i64,
    pub host: String,
    pub sequence: i64,
    pub fingerprint: String,
    pub tenant_id: String,
    pub ceiling: f64,
    pub status: ApprovalThresholdStatus,
}

impl ApprovalThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalThresholdStatus) -> bool {
        match self.status {
            ApprovalThresholdStatus::Created => {
                match next {
                    ApprovalThresholdStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalThresholdStatus::Validated => {
                match next {
                    ApprovalThresholdStatus::Published | ApprovalThresholdStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalThresholdStatus::Published => {
                match next {
                    ApprovalThresholdStatus::Retired => {
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

    pub fn move_to(&mut self, next: ApprovalThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval threshold", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalThreshold {
        ApprovalThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            window_days: self.window_days,
            host: self.host.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            tenant_id: self.tenant_id.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval threshold in status created.
pub fn new_approval_threshold(code: &str, label: &str) -> ApprovalThreshold {
    ApprovalThreshold {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        window_days: 0,
        host: "".to_string(),
        sequence: 0,
        fingerprint: "".to_string(),
        tenant_id: "".to_string(),
        ceiling: 0.0,
        status: ApprovalThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_threshold("code-719", "label-102");
        assert_eq!(item.status, ApprovalThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_approval_threshold("code-719", "label-102");
        if item.move_to(ApprovalThresholdStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalThresholdStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_threshold("code-719", "label-102");
        item.status = ApprovalThresholdStatus::Retired;
        assert!(
            item.move_to(ApprovalThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
