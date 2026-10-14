use super::*;

use acme_kit::text;

/// Where a primary failover preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryFailoverPreferenceStatus {
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

impl std::fmt::Display for PrimaryFailoverPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary failover preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreference {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub host: String,
    pub budget: f64,
    pub score: f64,
    pub source: String,
    pub revision: i64,
    pub audience: String,
    pub expires_at: i64,
    pub minimum: f64,
    pub status: PrimaryFailoverPreferenceStatus,
}

impl PrimaryFailoverPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverPreferenceStatus) -> bool {
        match self.status {
            PrimaryFailoverPreferenceStatus::Draft => {
                match next {
                    PrimaryFailoverPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverPreferenceStatus::Pending => {
                return next == PrimaryFailoverPreferenceStatus::Active
                    || next == PrimaryFailoverPreferenceStatus::Closed;
            }
            PrimaryFailoverPreferenceStatus::Active => {
                match next {
                    PrimaryFailoverPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverPreferenceStatus::Suspended => {
                return next == PrimaryFailoverPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryFailoverPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary failover preference", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverPreference {
        PrimaryFailoverPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            host: self.host.clone(),
            budget: self.budget,
            score: self.score,
            source: self.source.clone(),
            revision: self.revision,
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary failover preference in status draft.
pub fn new_primary_failover_preference(code: &str, title: &str) -> PrimaryFailoverPreference {
    PrimaryFailoverPreference {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        host: "".to_string(),
        budget: 0.0,
        score: 0.0,
        source: "".to_string(),
        revision: 0,
        audience: "".to_string(),
        expires_at: 0,
        minimum: 0.0,
        status: PrimaryFailoverPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_failover_preference("code-554", "title-363");
        assert_eq!(item.status, PrimaryFailoverPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_primary_failover_preference("code-554", "title-363");
        if item.move_to(PrimaryFailoverPreferenceStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFailoverPreferenceStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_failover_preference("code-554", "title-363");
        item.status = PrimaryFailoverPreferenceStatus::Closed;
        assert!(
            item.move_to(PrimaryFailoverPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
