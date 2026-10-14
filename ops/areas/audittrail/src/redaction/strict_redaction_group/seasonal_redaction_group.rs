use super::*;

use acme_kit::text;

/// Where a seasonal redaction group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRedactionGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalRedactionGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalRedactionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal redaction group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroup {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub scope: String,
    pub country: String,
    pub grace_days: i64,
    pub batch_size: i64,
    pub approved: bool,
    pub segment: String,
    pub sequence: i64,
    pub locale: String,
    pub status: SeasonalRedactionGroupStatus,
}

impl SeasonalRedactionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRedactionGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRedactionGroupStatus) -> bool {
        match self.status {
            SeasonalRedactionGroupStatus::Requested => {
                match next {
                    SeasonalRedactionGroupStatus::Granted | SeasonalRedactionGroupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRedactionGroupStatus::Granted => {
                match next {
                    SeasonalRedactionGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRedactionGroupStatus::Revoked => {
                return next == SeasonalRedactionGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRedactionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal redaction group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRedactionGroup {
        SeasonalRedactionGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            scope: self.scope.clone(),
            country: self.country.clone(),
            grace_days: self.grace_days,
            batch_size: self.batch_size,
            approved: self.approved,
            segment: self.segment.clone(),
            sequence: self.sequence,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal redaction group in status requested.
pub fn new_seasonal_redaction_group(code: &str, name: &str) -> SeasonalRedactionGroup {
    SeasonalRedactionGroup {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        scope: "".to_string(),
        country: "".to_string(),
        grace_days: 0,
        batch_size: 0,
        approved: false,
        segment: "".to_string(),
        sequence: 0,
        locale: "".to_string(),
        status: SeasonalRedactionGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_redaction_group("code-288", "name-980");
        assert_eq!(item.status, SeasonalRedactionGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_redaction_group("code-288", "name-980");
        if item.move_to(SeasonalRedactionGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRedactionGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_redaction_group("code-288", "name-980");
        item.status = SeasonalRedactionGroupStatus::Expired;
        assert!(
            item.move_to(SeasonalRedactionGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
