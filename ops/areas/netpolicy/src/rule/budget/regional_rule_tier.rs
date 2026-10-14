use super::*;

use acme_kit::text;

/// Where a regional rule tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRuleTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRuleTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRuleTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional rule tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTier {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub namespace: String,
    pub ratio: f64,
    pub status: RegionalRuleTierStatus,
}

impl RegionalRuleTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRuleTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRuleTierStatus) -> bool {
        match self.status {
            RegionalRuleTierStatus::Proposed => {
                return next == RegionalRuleTierStatus::Approved;
            }
            RegionalRuleTierStatus::Approved => {
                match next {
                    RegionalRuleTierStatus::Rejected | RegionalRuleTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRuleTierStatus::Rejected => {
                match next {
                    RegionalRuleTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalRuleTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rule tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional rule tier", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRuleTier {
        RegionalRuleTier {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional rule tier in status proposed.
pub fn new_regional_rule_tier(code: &str, title: &str) -> RegionalRuleTier {
    RegionalRuleTier {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        namespace: "".to_string(),
        ratio: 0.0,
        status: RegionalRuleTierStatus::Proposed,
    }
}
