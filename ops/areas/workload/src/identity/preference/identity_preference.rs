use super::*;

use acme_kit::text;

/// Where a identity preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityPreferenceStatus {
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

impl std::fmt::Display for IdentityPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPreference {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub notes: String,
    pub limit: i64,
    pub channel: String,
    pub budget: f64,
    pub rate: f64,
    pub status: IdentityPreferenceStatus,
}

impl IdentityPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityPreferenceStatus) -> bool {
        match self.status {
            IdentityPreferenceStatus::Draft => {
                return next == IdentityPreferenceStatus::Pending || next == IdentityPreferenceStatus::Suspended;
            }
            IdentityPreferenceStatus::Pending => {
                return next == IdentityPreferenceStatus::Active;
            }
            IdentityPreferenceStatus::Active => {
                match next {
                    IdentityPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityPreferenceStatus::Suspended => {
                match next {
                    IdentityPreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: IdentityPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.priority as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityPreference {
        IdentityPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            notes: self.notes.clone(),
            limit: self.limit,
            channel: self.channel.clone(),
            budget: self.budget,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a identity preference in status draft.
pub fn new_identity_preference(code: &str, name: &str) -> IdentityPreference {
    IdentityPreference {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        notes: "".to_string(),
        limit: 0,
        channel: "".to_string(),
        budget: 0.0,
        rate: 0.0,
        status: IdentityPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_identity_preference("code-786", "name-645");
        assert_eq!(item.status, IdentityPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_identity_preference("code-786", "name-645");
        if item.move_to(IdentityPreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityPreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_identity_preference("code-786", "name-645");
        item.status = IdentityPreferenceStatus::Closed;
        assert!(
            item.move_to(IdentityPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
