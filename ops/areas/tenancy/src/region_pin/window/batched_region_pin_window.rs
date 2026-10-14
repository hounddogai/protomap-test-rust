use super::*;

use acme_kit::text;

/// Where a batched region pin window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRegionPinWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedRegionPinWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedRegionPinWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched region pin window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindow {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub window_days: i64,
    pub summary: String,
    pub checksum: String,
    pub phone: String,
    pub issuer: String,
    pub rate: f64,
    pub description: String,
    pub pinned: bool,
    pub status: BatchedRegionPinWindowStatus,
}

impl BatchedRegionPinWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRegionPinWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRegionPinWindowStatus) -> bool {
        match self.status {
            BatchedRegionPinWindowStatus::Created => {
                match next {
                    BatchedRegionPinWindowStatus::Validated | BatchedRegionPinWindowStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRegionPinWindowStatus::Validated => {
                return next == BatchedRegionPinWindowStatus::Published;
            }
            BatchedRegionPinWindowStatus::Published => {
                match next {
                    BatchedRegionPinWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedRegionPinWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched region pin window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRegionPinWindow {
        BatchedRegionPinWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            window_days: self.window_days,
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            issuer: self.issuer.clone(),
            rate: self.rate,
            description: self.description.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched region pin window in status created.
pub fn new_batched_region_pin_window(code: &str, label: &str) -> BatchedRegionPinWindow {
    BatchedRegionPinWindow {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        window_days: 0,
        summary: "".to_string(),
        checksum: "".to_string(),
        phone: "".to_string(),
        issuer: "".to_string(),
        rate: 0.0,
        description: "".to_string(),
        pinned: false,
        status: BatchedRegionPinWindowStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_region_pin_window("code-76", "label-417");
        assert_eq!(item.status, BatchedRegionPinWindowStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_region_pin_window("code-76", "label-417");
        if item.move_to(BatchedRegionPinWindowStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRegionPinWindowStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_region_pin_window("code-76", "label-417");
        item.status = BatchedRegionPinWindowStatus::Retired;
        assert!(
            item.move_to(BatchedRegionPinWindowStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
