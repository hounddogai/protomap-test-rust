use super::*;

use acme_kit::text;

/// Where a regional usage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalUsageTierStatus {
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

impl std::fmt::Display for RegionalUsageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional usage tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTier {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub version: i64,
    pub ratio: f64,
    pub status: RegionalUsageTierStatus,
}

impl RegionalUsageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageTierStatus) -> bool {
        match self.status {
            RegionalUsageTierStatus::Draft => {
                return next == RegionalUsageTierStatus::Pending || next == RegionalUsageTierStatus::Suspended;
            }
            RegionalUsageTierStatus::Pending => {
                return next == RegionalUsageTierStatus::Active || next == RegionalUsageTierStatus::Suspended;
            }
            RegionalUsageTierStatus::Active => {
                match next {
                    RegionalUsageTierStatus::Suspended | RegionalUsageTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageTierStatus::Suspended => {
                return next == RegionalUsageTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalUsageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageTier {
        RegionalUsageTier {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            version: self.version,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional usage tier in status draft.
pub fn new_regional_usage_tier(code: &str, name: &str) -> RegionalUsageTier {
    RegionalUsageTier {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        version: 0,
        ratio: 0.0,
        status: RegionalUsageTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_usage_tier("code-787", "name-468");
        assert_eq!(item.status, RegionalUsageTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_regional_usage_tier("code-787", "name-468");
        if item.move_to(RegionalUsageTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_usage_tier("code-787", "name-468");
        item.status = RegionalUsageTierStatus::Closed;
        assert!(
            item.move_to(RegionalUsageTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
