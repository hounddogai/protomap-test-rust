use super::*;

use acme_kit::text;

/// Where a regional allowlist quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAllowlistQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalAllowlistQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalAllowlistQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional allowlist quota as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistQuota {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub batch_size: i64,
    pub quantity: i64,
    pub status: RegionalAllowlistQuotaStatus,
}

impl RegionalAllowlistQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAllowlistQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAllowlistQuotaStatus) -> bool {
        match self.status {
            RegionalAllowlistQuotaStatus::Proposed => {
                return next == RegionalAllowlistQuotaStatus::Withdrawn;
            }
            RegionalAllowlistQuotaStatus::Approved => {
                return next == RegionalAllowlistQuotaStatus::Rejected;
            }
            RegionalAllowlistQuotaStatus::Rejected => {
                return next == RegionalAllowlistQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalAllowlistQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional allowlist quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.batch_size as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAllowlistQuota {
        RegionalAllowlistQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            batch_size: self.batch_size,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional allowlist quota in status proposed.
pub fn new_regional_allowlist_quota(code: &str, title: &str) -> RegionalAllowlistQuota {
    RegionalAllowlistQuota {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        batch_size: 0,
        quantity: 0,
        status: RegionalAllowlistQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_allowlist_quota("code-935", "title-165");
        assert_eq!(item.status, RegionalAllowlistQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_regional_allowlist_quota("code-935", "title-165");
        if item.move_to(RegionalAllowlistQuotaStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAllowlistQuotaStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_allowlist_quota("code-935", "title-165");
        item.status = RegionalAllowlistQuotaStatus::Withdrawn;
        assert!(
            item.move_to(RegionalAllowlistQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
