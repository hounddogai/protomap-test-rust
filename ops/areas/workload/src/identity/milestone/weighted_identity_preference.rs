use super::*;

use acme_kit::text;

/// Where a weighted identity preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedIdentityPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedIdentityPreferenceStatus {
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

impl std::fmt::Display for WeightedIdentityPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted identity preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreference {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub principal: String,
    pub approved: bool,
    pub source: String,
    pub status: WeightedIdentityPreferenceStatus,
}

impl WeightedIdentityPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedIdentityPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedIdentityPreferenceStatus) -> bool {
        match self.status {
            WeightedIdentityPreferenceStatus::Draft => {
                return next == WeightedIdentityPreferenceStatus::Suspended;
            }
            WeightedIdentityPreferenceStatus::Pending => {
                return next == WeightedIdentityPreferenceStatus::Active
                    || next == WeightedIdentityPreferenceStatus::Suspended;
            }
            WeightedIdentityPreferenceStatus::Active => {
                match next {
                    WeightedIdentityPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityPreferenceStatus::Suspended => {
                return next == WeightedIdentityPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedIdentityPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted identity preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted identity preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> WeightedIdentityPreference {
        WeightedIdentityPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            principal: self.principal.clone(),
            approved: self.approved,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted identity preference in status draft.
pub fn new_weighted_identity_preference(code: &str, name: &str) -> WeightedIdentityPreference {
    WeightedIdentityPreference {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        principal: "".to_string(),
        approved: false,
        source: "".to_string(),
        status: WeightedIdentityPreferenceStatus::Draft,
    }
}
