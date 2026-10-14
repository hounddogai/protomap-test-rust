use super::*;

use acme_kit::text;

/// Where a global nonce estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalNonceEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalNonceEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalNonceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global nonce estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimate {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub priority: i64,
    pub limit: i64,
    pub contact_email: String,
    pub external_ref: String,
    pub principal: String,
    pub visible: bool,
    pub status: GlobalNonceEstimateStatus,
}

impl GlobalNonceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalNonceEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalNonceEstimateStatus) -> bool {
        match self.status {
            GlobalNonceEstimateStatus::Created => {
                match next {
                    GlobalNonceEstimateStatus::Validated | GlobalNonceEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalNonceEstimateStatus::Validated => {
                match next {
                    GlobalNonceEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalNonceEstimateStatus::Published => {
                return next == GlobalNonceEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalNonceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global nonce estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.limit as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalNonceEstimate {
        GlobalNonceEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            priority: self.priority,
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global nonce estimate in status created.
pub fn new_global_nonce_estimate(code: &str, name: &str) -> GlobalNonceEstimate {
    GlobalNonceEstimate {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        priority: 0,
        limit: 0,
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        principal: "".to_string(),
        visible: false,
        status: GlobalNonceEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_nonce_estimate("code-534", "name-334");
        assert_eq!(item.status, GlobalNonceEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_global_nonce_estimate("code-534", "name-334");
        if item.move_to(GlobalNonceEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalNonceEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_nonce_estimate("code-534", "name-334");
        item.status = GlobalNonceEstimateStatus::Retired;
        assert!(
            item.move_to(GlobalNonceEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
