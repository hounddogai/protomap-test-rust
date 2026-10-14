use super::*;

use acme_kit::text;

/// Where a freeze profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FreezeProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FreezeProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProfile {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub amount: f64,
    pub starts_at: i64,
    pub status: FreezeProfileStatus,
}

impl FreezeProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeProfileStatus) -> bool {
        match self.status {
            FreezeProfileStatus::Created => {
                return next == FreezeProfileStatus::Published || next == FreezeProfileStatus::Retired;
            }
            FreezeProfileStatus::Validated => {
                return next == FreezeProfileStatus::Published || next == FreezeProfileStatus::Retired;
            }
            FreezeProfileStatus::Published => {
                return next == FreezeProfileStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeProfile {
        FreezeProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            amount: self.amount,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze profile in status created.
pub fn new_freeze_profile(code: &str, name: &str) -> FreezeProfile {
    FreezeProfile {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        amount: 0.0,
        starts_at: 0,
        status: FreezeProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_freeze_profile("code-728", "name-445");
        assert_eq!(item.status, FreezeProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_freeze_profile("code-728", "name-445");
        if item.move_to(FreezeProfileStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeProfileStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_freeze_profile("code-728", "name-445");
        item.status = FreezeProfileStatus::Retired;
        assert!(
            item.move_to(FreezeProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
