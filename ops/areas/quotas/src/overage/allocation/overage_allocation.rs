use super::*;

use acme_kit::text;

/// Where a overage allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl OverageAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for OverageAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocation {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub window_days: i64,
    pub cipher: String,
    pub status: OverageAllocationStatus,
}

impl OverageAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageAllocationStatus) -> bool {
        match self.status {
            OverageAllocationStatus::Created => {
                match next {
                    OverageAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAllocationStatus::Validated => {
                match next {
                    OverageAllocationStatus::Published | OverageAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAllocationStatus::Published => {
                return next == OverageAllocationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage allocation", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.window_days as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageAllocation {
        OverageAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            window_days: self.window_days,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a overage allocation in status created.
pub fn new_overage_allocation(code: &str, label: &str) -> OverageAllocation {
    OverageAllocation {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        window_days: 0,
        cipher: "".to_string(),
        status: OverageAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_overage_allocation("code-771", "label-991");
        assert_eq!(item.status, OverageAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_overage_allocation("code-771", "label-991");
        if item.move_to(OverageAllocationStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageAllocationStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_overage_allocation("code-771", "label-991");
        item.status = OverageAllocationStatus::Retired;
        assert!(
            item.move_to(OverageAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
