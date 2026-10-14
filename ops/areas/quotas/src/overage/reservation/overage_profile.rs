use super::*;

use acme_kit::text;

/// Where a overage profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl OverageProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for OverageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfile {
    pub code: String,
    pub label: String,
    pub region: String,
    pub sequence: i64,
    pub tenant_id: String,
    pub status: OverageProfileStatus,
}

impl OverageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageProfileStatus) -> bool {
        match self.status {
            OverageProfileStatus::Created => {
                match next {
                    OverageProfileStatus::Validated | OverageProfileStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageProfileStatus::Validated => {
                match next {
                    OverageProfileStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageProfileStatus::Published => {
                match next {
                    OverageProfileStatus::Retired => {
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

    pub fn move_to(&mut self, next: OverageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> OverageProfile {
        OverageProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            sequence: self.sequence,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a overage profile in status created.
pub fn new_overage_profile(code: &str, label: &str) -> OverageProfile {
    OverageProfile {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        sequence: 0,
        tenant_id: "".to_string(),
        status: OverageProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_overage_profile("code-699", "label-54");
        assert_eq!(item.status, OverageProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_overage_profile("code-699", "label-54");
        if item.move_to(OverageProfileStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageProfileStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_overage_profile("code-699", "label-54");
        item.status = OverageProfileStatus::Retired;
        assert!(
            item.move_to(OverageProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
