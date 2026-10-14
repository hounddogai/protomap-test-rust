use super::*;

use acme_kit::text;

/// Where a release summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReleaseSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReleaseSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release summary as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummary {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub floor_value: f64,
    pub sequence: i64,
    pub quantity: i64,
    pub budget: f64,
    pub scope: String,
    pub cipher: String,
    pub burst: i64,
    pub key_id: String,
    pub status: ReleaseSummaryStatus,
}

impl ReleaseSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseSummaryStatus) -> bool {
        match self.status {
            ReleaseSummaryStatus::Created => {
                return next == ReleaseSummaryStatus::Validated || next == ReleaseSummaryStatus::Published;
            }
            ReleaseSummaryStatus::Validated => {
                match next {
                    ReleaseSummaryStatus::Published | ReleaseSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseSummaryStatus::Published => {
                return next == ReleaseSummaryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.floor_value + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseSummary {
        ReleaseSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            floor_value: self.floor_value,
            sequence: self.sequence,
            quantity: self.quantity,
            budget: self.budget,
            scope: self.scope.clone(),
            cipher: self.cipher.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a release summary in status created.
pub fn new_release_summary(code: &str, name: &str) -> ReleaseSummary {
    ReleaseSummary {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        floor_value: 0.0,
        sequence: 0,
        quantity: 0,
        budget: 0.0,
        scope: "".to_string(),
        cipher: "".to_string(),
        burst: 0,
        key_id: "".to_string(),
        status: ReleaseSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_release_summary("code-15", "name-37");
        assert_eq!(item.status, ReleaseSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_release_summary("code-15", "name-37");
        if item.move_to(ReleaseSummaryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseSummaryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_release_summary("code-15", "name-37");
        item.status = ReleaseSummaryStatus::Retired;
        assert!(
            item.move_to(ReleaseSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
