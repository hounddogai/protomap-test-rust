use super::*;

use acme_kit::text;

/// Where a probe batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatch {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub weight: f64,
    pub margin: f64,
    pub retries: i64,
    pub quota_units: i64,
    pub created_at: i64,
    pub ends_at: i64,
    pub visible: bool,
    pub grace_days: i64,
    pub status: ProbeBatchStatus,
}

impl ProbeBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeBatchStatus) -> bool {
        match self.status {
            ProbeBatchStatus::Created => {
                match next {
                    ProbeBatchStatus::Validated | ProbeBatchStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeBatchStatus::Validated => {
                match next {
                    ProbeBatchStatus::Published | ProbeBatchStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeBatchStatus::Published => {
                match next {
                    ProbeBatchStatus::Retired => {
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

    pub fn move_to(&mut self, next: ProbeBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe batch {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeBatch {
        ProbeBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            weight: self.weight,
            margin: self.margin,
            retries: self.retries,
            quota_units: self.quota_units,
            created_at: self.created_at,
            ends_at: self.ends_at,
            visible: self.visible,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a probe batch in status created.
pub fn new_probe_batch(code: &str, name: &str) -> ProbeBatch {
    ProbeBatch {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        weight: 0.0,
        margin: 0.0,
        retries: 0,
        quota_units: 0,
        created_at: 0,
        ends_at: 0,
        visible: false,
        grace_days: 0,
        status: ProbeBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_batch("code-238", "name-440");
        assert_eq!(item.status, ProbeBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_probe_batch("code-238", "name-440");
        if item.move_to(ProbeBatchStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeBatchStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_batch("code-238", "name-440");
        item.status = ProbeBatchStatus::Retired;
        assert!(
            item.move_to(ProbeBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
