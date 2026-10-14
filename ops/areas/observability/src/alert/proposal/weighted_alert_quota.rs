use super::*;

use acme_kit::text;

/// Where a weighted alert quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedAlertQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedAlertQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted alert quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuota {
    pub code: String,
    pub label: String,
    pub region: String,
    pub amount: f64,
    pub contact_email: String,
    pub status: WeightedAlertQuotaStatus,
}

impl WeightedAlertQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertQuotaStatus) -> bool {
        match self.status {
            WeightedAlertQuotaStatus::Created => {
                return next == WeightedAlertQuotaStatus::Published || next == WeightedAlertQuotaStatus::Retired;
            }
            WeightedAlertQuotaStatus::Validated => {
                match next {
                    WeightedAlertQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertQuotaStatus::Published => {
                return next == WeightedAlertQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedAlertQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> WeightedAlertQuota {
        WeightedAlertQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted alert quota in status created.
pub fn new_weighted_alert_quota(code: &str, label: &str) -> WeightedAlertQuota {
    WeightedAlertQuota {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        amount: 0.0,
        contact_email: "".to_string(),
        status: WeightedAlertQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_alert_quota("code-747", "label-966");
        assert_eq!(item.status, WeightedAlertQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_weighted_alert_quota("code-747", "label-966");
        if item.move_to(WeightedAlertQuotaStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertQuotaStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_alert_quota("code-747", "label-966");
        item.status = WeightedAlertQuotaStatus::Retired;
        assert!(
            item.move_to(WeightedAlertQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
