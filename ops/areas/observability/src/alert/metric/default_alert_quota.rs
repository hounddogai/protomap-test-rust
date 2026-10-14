use super::*;

use acme_kit::text;

/// Where a default alert quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAlertQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultAlertQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultAlertQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default alert quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertQuota {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub budget: f64,
    pub amount: f64,
    pub status: DefaultAlertQuotaStatus,
}

impl DefaultAlertQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAlertQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAlertQuotaStatus) -> bool {
        match self.status {
            DefaultAlertQuotaStatus::Created => {
                return next == DefaultAlertQuotaStatus::Retired;
            }
            DefaultAlertQuotaStatus::Validated => {
                return next == DefaultAlertQuotaStatus::Published || next == DefaultAlertQuotaStatus::Retired;
            }
            DefaultAlertQuotaStatus::Published => {
                match next {
                    DefaultAlertQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: DefaultAlertQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default alert quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.amount + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultAlertQuota {
        DefaultAlertQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            budget: self.budget,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default alert quota in status created.
pub fn new_default_alert_quota(code: &str, title: &str) -> DefaultAlertQuota {
    DefaultAlertQuota {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        budget: 0.0,
        amount: 0.0,
        status: DefaultAlertQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_alert_quota("code-197", "title-118");
        assert_eq!(item.status, DefaultAlertQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_default_alert_quota("code-197", "title-118");
        if item.move_to(DefaultAlertQuotaStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAlertQuotaStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_alert_quota("code-197", "title-118");
        item.status = DefaultAlertQuotaStatus::Retired;
        assert!(
            item.move_to(DefaultAlertQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
