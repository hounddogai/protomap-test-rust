use super::*;

use acme_kit::text;

/// Where a failover variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVariance {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub phone: String,
    pub country: String,
    pub tags: Vec<String>,
    pub status: FailoverVarianceStatus,
}

impl FailoverVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverVarianceStatus) -> bool {
        match self.status {
            FailoverVarianceStatus::Proposed => {
                match next {
                    FailoverVarianceStatus::Approved | FailoverVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverVarianceStatus::Approved => {
                return next == FailoverVarianceStatus::Withdrawn;
            }
            FailoverVarianceStatus::Rejected => {
                return next == FailoverVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> FailoverVariance {
        FailoverVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            country: self.country.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a failover variance in status proposed.
pub fn new_failover_variance(code: &str, name: &str) -> FailoverVariance {
    FailoverVariance {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        phone: "".to_string(),
        country: "".to_string(),
        tags: Vec::new(),
        status: FailoverVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_failover_variance("code-830", "name-909");
        assert_eq!(item.status, FailoverVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_failover_variance("code-830", "name-909");
        if item.move_to(FailoverVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_failover_variance("code-830", "name-909");
        item.status = FailoverVarianceStatus::Withdrawn;
        assert!(
            item.move_to(FailoverVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
