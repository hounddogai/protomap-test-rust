use super::*;

use acme_kit::text;

/// Where a legacy failover cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFailoverCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyFailoverCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyFailoverCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy failover cycle as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycle {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub host: String,
    pub batch_size: i64,
    pub budget: f64,
    pub ratio: f64,
    pub currency: String,
    pub audience: String,
    pub status: LegacyFailoverCycleStatus,
}

impl LegacyFailoverCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFailoverCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFailoverCycleStatus) -> bool {
        match self.status {
            LegacyFailoverCycleStatus::Created => {
                match next {
                    LegacyFailoverCycleStatus::Validated | LegacyFailoverCycleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverCycleStatus::Validated => {
                return next == LegacyFailoverCycleStatus::Published;
            }
            LegacyFailoverCycleStatus::Published => {
                match next {
                    LegacyFailoverCycleStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyFailoverCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy failover cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy failover cycle", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.ratio * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFailoverCycle {
        LegacyFailoverCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            host: self.host.clone(),
            batch_size: self.batch_size,
            budget: self.budget,
            ratio: self.ratio,
            currency: self.currency.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy failover cycle in status created.
pub fn new_legacy_failover_cycle(code: &str, title: &str) -> LegacyFailoverCycle {
    LegacyFailoverCycle {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        host: "".to_string(),
        batch_size: 0,
        budget: 0.0,
        ratio: 0.0,
        currency: "".to_string(),
        audience: "".to_string(),
        status: LegacyFailoverCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_failover_cycle("code-208", "title-564");
        assert_eq!(item.status, LegacyFailoverCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_failover_cycle("code-208", "title-564");
        if item.move_to(LegacyFailoverCycleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFailoverCycleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_failover_cycle("code-208", "title-564");
        item.status = LegacyFailoverCycleStatus::Retired;
        assert!(
            item.move_to(LegacyFailoverCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
