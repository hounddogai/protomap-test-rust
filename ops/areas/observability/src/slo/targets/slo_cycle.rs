use super::*;

use acme_kit::text;

/// Where a slo cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SloCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SloCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloCycle {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub rotation_days: i64,
    pub updated_at: i64,
    pub status: SloCycleStatus,
}

impl SloCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloCycleStatus) -> bool {
        match self.status {
            SloCycleStatus::Created => {
                return next == SloCycleStatus::Retired;
            }
            SloCycleStatus::Validated => {
                return next == SloCycleStatus::Published || next == SloCycleStatus::Retired;
            }
            SloCycleStatus::Published => {
                return next == SloCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloCycle {
        SloCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo cycle in status created.
pub fn new_slo_cycle(code: &str, name: &str) -> SloCycle {
    SloCycle {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        rotation_days: 0,
        updated_at: 0,
        status: SloCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_slo_cycle("code-103", "name-188");
        assert_eq!(item.status, SloCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_slo_cycle("code-103", "name-188");
        if item.move_to(SloCycleStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloCycleStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_slo_cycle("code-103", "name-188");
        item.status = SloCycleStatus::Retired;
        assert!(item.move_to(SloCycleStatus::Created).is_err(), "expected move to fail");
    }
}
