use super::*;

use acme_kit::text;

/// Where a overage threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl OverageThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for OverageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageThreshold {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub score: f64,
    pub budget: f64,
    pub key_id: String,
    pub batch_size: i64,
    pub status: OverageThresholdStatus,
}

impl OverageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageThresholdStatus) -> bool {
        match self.status {
            OverageThresholdStatus::Created => {
                match next {
                    OverageThresholdStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageThresholdStatus::Validated => {
                match next {
                    OverageThresholdStatus::Published | OverageThresholdStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageThresholdStatus::Published => {
                return next == OverageThresholdStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "overage threshold", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.score + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageThreshold {
        OverageThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            score: self.score,
            budget: self.budget,
            key_id: self.key_id.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a overage threshold in status created.
pub fn new_overage_threshold(code: &str, name: &str) -> OverageThreshold {
    OverageThreshold {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        score: 0.0,
        budget: 0.0,
        key_id: "".to_string(),
        batch_size: 0,
        status: OverageThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_overage_threshold("code-240", "name-319");
        assert_eq!(item.status, OverageThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_overage_threshold("code-240", "name-319");
        if item.move_to(OverageThresholdStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageThresholdStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_overage_threshold("code-240", "name-319");
        item.status = OverageThresholdStatus::Retired;
        assert!(
            item.move_to(OverageThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
