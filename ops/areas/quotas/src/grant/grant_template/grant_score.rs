use super::*;

use acme_kit::text;

/// Where a grant score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant score as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantScore {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub port: i64,
    pub label: String,
    pub starts_at: i64,
    pub title: String,
    pub version: i64,
    pub created_at: i64,
    pub audience: String,
    pub status: GrantScoreStatus,
}

impl GrantScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantScoreStatus) -> bool {
        match self.status {
            GrantScoreStatus::Requested => {
                match next {
                    GrantScoreStatus::Granted | GrantScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantScoreStatus::Granted => {
                return next == GrantScoreStatus::Revoked || next == GrantScoreStatus::Expired;
            }
            GrantScoreStatus::Revoked => {
                return next == GrantScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.created_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantScore {
        GrantScore {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            port: self.port,
            label: self.label.clone(),
            starts_at: self.starts_at,
            title: self.title.clone(),
            version: self.version,
            created_at: self.created_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a grant score in status requested.
pub fn new_grant_score(code: &str, name: &str) -> GrantScore {
    GrantScore {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        port: 0,
        label: "".to_string(),
        starts_at: 0,
        title: "".to_string(),
        version: 0,
        created_at: 0,
        audience: "".to_string(),
        status: GrantScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_score("code-699", "name-536");
        assert_eq!(item.status, GrantScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_grant_score("code-699", "name-536");
        if item.move_to(GrantScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_score("code-699", "name-536");
        item.status = GrantScoreStatus::Expired;
        assert!(
            item.move_to(GrantScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
