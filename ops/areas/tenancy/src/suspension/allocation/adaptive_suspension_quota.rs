use super::*;

use acme_kit::text;

/// Where a adaptive suspension quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSuspensionQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveSuspensionQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveSuspensionQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive suspension quota as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuota {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub pinned: bool,
    pub amount: f64,
    pub scope: String,
    pub discount: f64,
    pub status: AdaptiveSuspensionQuotaStatus,
}

impl AdaptiveSuspensionQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSuspensionQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSuspensionQuotaStatus) -> bool {
        match self.status {
            AdaptiveSuspensionQuotaStatus::Draft => {
                match next {
                    AdaptiveSuspensionQuotaStatus::Pending | AdaptiveSuspensionQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSuspensionQuotaStatus::Pending => {
                return next == AdaptiveSuspensionQuotaStatus::Suspended
                    || next == AdaptiveSuspensionQuotaStatus::Closed;
            }
            AdaptiveSuspensionQuotaStatus::Active => {
                return next == AdaptiveSuspensionQuotaStatus::Closed;
            }
            AdaptiveSuspensionQuotaStatus::Suspended => {
                return next == AdaptiveSuspensionQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSuspensionQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive suspension quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.discount * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSuspensionQuota {
        AdaptiveSuspensionQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            amount: self.amount,
            scope: self.scope.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive suspension quota in status draft.
pub fn new_adaptive_suspension_quota(code: &str, label: &str) -> AdaptiveSuspensionQuota {
    AdaptiveSuspensionQuota {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        pinned: false,
        amount: 0.0,
        scope: "".to_string(),
        discount: 0.0,
        status: AdaptiveSuspensionQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_suspension_quota("code-638", "label-80");
        assert_eq!(item.status, AdaptiveSuspensionQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_adaptive_suspension_quota("code-638", "label-80");
        if item.move_to(AdaptiveSuspensionQuotaStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSuspensionQuotaStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_suspension_quota("code-638", "label-80");
        item.status = AdaptiveSuspensionQuotaStatus::Closed;
        assert!(
            item.move_to(AdaptiveSuspensionQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
