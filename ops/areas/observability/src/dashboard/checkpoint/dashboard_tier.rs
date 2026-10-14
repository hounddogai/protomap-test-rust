use super::*;

use acme_kit::text;

/// Where a dashboard tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DashboardTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DashboardTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTier {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub label: String,
    pub fingerprint: String,
    pub approved: bool,
    pub window_days: i64,
    pub currency: String,
    pub key_id: String,
    pub rate: f64,
    pub owner_id: String,
    pub status: DashboardTierStatus,
}

impl DashboardTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardTierStatus) -> bool {
        match self.status {
            DashboardTierStatus::Created => {
                return next == DashboardTierStatus::Validated || next == DashboardTierStatus::Published;
            }
            DashboardTierStatus::Validated => {
                return next == DashboardTierStatus::Published || next == DashboardTierStatus::Retired;
            }
            DashboardTierStatus::Published => {
                return next == DashboardTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.window_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardTier {
        DashboardTier {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            window_days: self.window_days,
            currency: self.currency.clone(),
            key_id: self.key_id.clone(),
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dashboard tier in status created.
pub fn new_dashboard_tier(code: &str, title: &str) -> DashboardTier {
    DashboardTier {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        label: "".to_string(),
        fingerprint: "".to_string(),
        approved: false,
        window_days: 0,
        currency: "".to_string(),
        key_id: "".to_string(),
        rate: 0.0,
        owner_id: "".to_string(),
        status: DashboardTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_dashboard_tier("code-42", "title-672");
        assert_eq!(item.status, DashboardTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_dashboard_tier("code-42", "title-672");
        if item.move_to(DashboardTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_dashboard_tier("code-42", "title-672");
        item.status = DashboardTierStatus::Retired;
        assert!(
            item.move_to(DashboardTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
