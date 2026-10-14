use super::*;

use acme_kit::text;

/// Where a regional usage score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalUsageScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalUsageScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional usage score as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScore {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub notes: String,
    pub discount: f64,
    pub limit: i64,
    pub weight: f64,
    pub segment: String,
    pub revision: i64,
    pub timezone: String,
    pub status: RegionalUsageScoreStatus,
}

impl RegionalUsageScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageScoreStatus) -> bool {
        match self.status {
            RegionalUsageScoreStatus::Created => {
                return next == RegionalUsageScoreStatus::Published;
            }
            RegionalUsageScoreStatus::Validated => {
                return next == RegionalUsageScoreStatus::Retired;
            }
            RegionalUsageScoreStatus::Published => {
                return next == RegionalUsageScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalUsageScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional usage score", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageScore {
        RegionalUsageScore {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            discount: self.discount,
            limit: self.limit,
            weight: self.weight,
            segment: self.segment.clone(),
            revision: self.revision,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional usage score in status created.
pub fn new_regional_usage_score(code: &str, title: &str) -> RegionalUsageScore {
    RegionalUsageScore {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        discount: 0.0,
        limit: 0,
        weight: 0.0,
        segment: "".to_string(),
        revision: 0,
        timezone: "".to_string(),
        status: RegionalUsageScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_usage_score("code-298", "title-335");
        assert_eq!(item.status, RegionalUsageScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_usage_score("code-298", "title-335");
        if item.move_to(RegionalUsageScoreStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageScoreStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_usage_score("code-298", "title-335");
        item.status = RegionalUsageScoreStatus::Retired;
        assert!(
            item.move_to(RegionalUsageScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
