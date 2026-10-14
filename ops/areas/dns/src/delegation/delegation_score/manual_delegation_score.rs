use super::*;

use acme_kit::text;

/// Where a manual delegation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDelegationScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualDelegationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualDelegationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual delegation score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScore {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub scope: String,
    pub ceiling: f64,
    pub limit: i64,
    pub contact_email: String,
    pub ends_at: i64,
    pub description: String,
    pub channel: String,
    pub window_days: i64,
    pub status: ManualDelegationScoreStatus,
}

impl ManualDelegationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDelegationScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDelegationScoreStatus) -> bool {
        match self.status {
            ManualDelegationScoreStatus::Created => {
                match next {
                    ManualDelegationScoreStatus::Validated | ManualDelegationScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDelegationScoreStatus::Validated => {
                match next {
                    ManualDelegationScoreStatus::Published | ManualDelegationScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDelegationScoreStatus::Published => {
                match next {
                    ManualDelegationScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualDelegationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual delegation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual delegation score", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDelegationScore {
        ManualDelegationScore {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            ends_at: self.ends_at,
            description: self.description.clone(),
            channel: self.channel.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual delegation score in status created.
pub fn new_manual_delegation_score(code: &str, name: &str) -> ManualDelegationScore {
    ManualDelegationScore {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        scope: "".to_string(),
        ceiling: 0.0,
        limit: 0,
        contact_email: "".to_string(),
        ends_at: 0,
        description: "".to_string(),
        channel: "".to_string(),
        window_days: 0,
        status: ManualDelegationScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_delegation_score("code-437", "name-270");
        assert_eq!(item.status, ManualDelegationScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_delegation_score("code-437", "name-270");
        if item.move_to(ManualDelegationScoreStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDelegationScoreStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_delegation_score("code-437", "name-270");
        item.status = ManualDelegationScoreStatus::Retired;
        assert!(
            item.move_to(ManualDelegationScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
