use super::*;

use acme_kit::text;

/// Where a isolation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimate {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub external_ref: String,
    pub channel: String,
    pub attempts: i64,
    pub contact_email: String,
    pub unit: String,
    pub status: IsolationEstimateStatus,
}

impl IsolationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationEstimateStatus) -> bool {
        match self.status {
            IsolationEstimateStatus::Created => {
                return next == IsolationEstimateStatus::Published || next == IsolationEstimateStatus::Retired;
            }
            IsolationEstimateStatus::Validated => {
                return next == IsolationEstimateStatus::Published;
            }
            IsolationEstimateStatus::Published => {
                return next == IsolationEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.threshold * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationEstimate {
        IsolationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a isolation estimate in status created.
pub fn new_isolation_estimate(code: &str, label: &str) -> IsolationEstimate {
    IsolationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        external_ref: "".to_string(),
        channel: "".to_string(),
        attempts: 0,
        contact_email: "".to_string(),
        unit: "".to_string(),
        status: IsolationEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_estimate("code-774", "label-807");
        assert_eq!(item.status, IsolationEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_isolation_estimate("code-774", "label-807");
        if item.move_to(IsolationEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_estimate("code-774", "label-807");
        item.status = IsolationEstimateStatus::Retired;
        assert!(
            item.move_to(IsolationEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
