use super::*;

use acme_kit::text;

/// Where a weighted limit tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedLimitTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedLimitTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for WeightedLimitTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted limit tier as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTier {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub label: String,
    pub audience: String,
    pub discount: f64,
    pub owner_id: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub country: String,
    pub status: WeightedLimitTierStatus,
}

impl WeightedLimitTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedLimitTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedLimitTierStatus) -> bool {
        match self.status {
            WeightedLimitTierStatus::Open => {
                match next {
                    WeightedLimitTierStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedLimitTierStatus::InReview => {
                match next {
                    WeightedLimitTierStatus::Reopened | WeightedLimitTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedLimitTierStatus::Resolved => {
                match next {
                    WeightedLimitTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedLimitTierStatus::Reopened => {
                match next {
                    WeightedLimitTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedLimitTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted limit tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.maximum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedLimitTier {
        WeightedLimitTier {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            label: self.label.clone(),
            audience: self.audience.clone(),
            discount: self.discount,
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted limit tier in status open.
pub fn new_weighted_limit_tier(code: &str, name: &str) -> WeightedLimitTier {
    WeightedLimitTier {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        label: "".to_string(),
        audience: "".to_string(),
        discount: 0.0,
        owner_id: "".to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        contact_email: "".to_string(),
        country: "".to_string(),
        status: WeightedLimitTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_limit_tier("code-921", "name-264");
        assert_eq!(item.status, WeightedLimitTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_weighted_limit_tier("code-921", "name-264");
        if item.move_to(WeightedLimitTierStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedLimitTierStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_limit_tier("code-921", "name-264");
        item.status = WeightedLimitTierStatus::Archived;
        assert!(
            item.move_to(WeightedLimitTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
