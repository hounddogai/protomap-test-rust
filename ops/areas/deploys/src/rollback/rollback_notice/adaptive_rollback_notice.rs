use super::*;

use acme_kit::text;

/// Where a adaptive rollback notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRollbackNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveRollbackNoticeStatus {
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

impl std::fmt::Display for AdaptiveRollbackNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollback notice as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNotice {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub discount: f64,
    pub amount: f64,
    pub weight: f64,
    pub rotation_days: i64,
    pub source: String,
    pub minimum: f64,
    pub window_days: i64,
    pub retries: i64,
    pub status: AdaptiveRollbackNoticeStatus,
}

impl AdaptiveRollbackNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRollbackNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRollbackNoticeStatus) -> bool {
        match self.status {
            AdaptiveRollbackNoticeStatus::Draft => {
                return next == AdaptiveRollbackNoticeStatus::Pending || next == AdaptiveRollbackNoticeStatus::Closed;
            }
            AdaptiveRollbackNoticeStatus::Pending => {
                return next == AdaptiveRollbackNoticeStatus::Closed;
            }
            AdaptiveRollbackNoticeStatus::Active => {
                return next == AdaptiveRollbackNoticeStatus::Suspended;
            }
            AdaptiveRollbackNoticeStatus::Suspended => {
                return next == AdaptiveRollbackNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRollbackNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollback notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRollbackNotice {
        AdaptiveRollbackNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            discount: self.discount,
            amount: self.amount,
            weight: self.weight,
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            minimum: self.minimum,
            window_days: self.window_days,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive rollback notice in status draft.
pub fn new_adaptive_rollback_notice(code: &str, label: &str) -> AdaptiveRollbackNotice {
    AdaptiveRollbackNotice {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        discount: 0.0,
        amount: 0.0,
        weight: 0.0,
        rotation_days: 0,
        source: "".to_string(),
        minimum: 0.0,
        window_days: 0,
        retries: 0,
        status: AdaptiveRollbackNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_rollback_notice("code-130", "label-160");
        assert_eq!(item.status, AdaptiveRollbackNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_adaptive_rollback_notice("code-130", "label-160");
        if item.move_to(AdaptiveRollbackNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRollbackNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_rollback_notice("code-130", "label-160");
        item.status = AdaptiveRollbackNoticeStatus::Closed;
        assert!(
            item.move_to(AdaptiveRollbackNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
