use super::*;

use acme_kit::text;

/// Where a fallback action item notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackActionItemNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackActionItemNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackActionItemNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback action item notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNotice {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub unit: String,
    pub currency: String,
    pub active: bool,
    pub priority: i64,
    pub minimum: f64,
    pub limit: i64,
    pub ends_at: i64,
    pub fingerprint: String,
    pub status: FallbackActionItemNoticeStatus,
}

impl FallbackActionItemNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackActionItemNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackActionItemNoticeStatus) -> bool {
        match self.status {
            FallbackActionItemNoticeStatus::Created => {
                match next {
                    FallbackActionItemNoticeStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemNoticeStatus::Validated => {
                match next {
                    FallbackActionItemNoticeStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemNoticeStatus::Published => {
                return next == FallbackActionItemNoticeStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackActionItemNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback action item notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.minimum * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackActionItemNotice {
        FallbackActionItemNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            active: self.active,
            priority: self.priority,
            minimum: self.minimum,
            limit: self.limit,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback action item notice in status created.
pub fn new_fallback_action_item_notice(code: &str, name: &str) -> FallbackActionItemNotice {
    FallbackActionItemNotice {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        active: false,
        priority: 0,
        minimum: 0.0,
        limit: 0,
        ends_at: 0,
        fingerprint: "".to_string(),
        status: FallbackActionItemNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_action_item_notice("code-607", "name-857");
        assert_eq!(item.status, FallbackActionItemNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_action_item_notice("code-607", "name-857");
        if item.move_to(FallbackActionItemNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackActionItemNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_action_item_notice("code-607", "name-857");
        item.status = FallbackActionItemNoticeStatus::Retired;
        assert!(
            item.move_to(FallbackActionItemNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
