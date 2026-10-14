use super::*;

use acme_kit::text;

/// Where a tiered limit target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredLimitTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredLimitTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredLimitTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered limit target as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTarget {
    pub code: String,
    pub name: String,
    pub country: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub status: TieredLimitTargetStatus,
}

impl TieredLimitTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredLimitTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredLimitTargetStatus) -> bool {
        match self.status {
            TieredLimitTargetStatus::Created => {
                match next {
                    TieredLimitTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredLimitTargetStatus::Validated => {
                return next == TieredLimitTargetStatus::Published;
            }
            TieredLimitTargetStatus::Published => {
                match next {
                    TieredLimitTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredLimitTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered limit target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredLimitTarget {
        TieredLimitTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered limit target in status created.
pub fn new_tiered_limit_target(code: &str, name: &str) -> TieredLimitTarget {
    TieredLimitTarget {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        port: 0,
        ttl_seconds: 0,
        status: TieredLimitTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_limit_target("code-22", "name-433");
        assert_eq!(item.status, TieredLimitTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_tiered_limit_target("code-22", "name-433");
        if item.move_to(TieredLimitTargetStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredLimitTargetStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_limit_target("code-22", "name-433");
        item.status = TieredLimitTargetStatus::Retired;
        assert!(
            item.move_to(TieredLimitTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
