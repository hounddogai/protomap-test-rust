use super::*;

use acme_kit::text;

/// Where a regional grant estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalGrantEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalGrantEstimateStatus {
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

impl std::fmt::Display for RegionalGrantEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional grant estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantEstimate {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub name: String,
    pub timezone: String,
    pub tags: Vec<String>,
    pub status: RegionalGrantEstimateStatus,
}

impl RegionalGrantEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalGrantEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalGrantEstimateStatus) -> bool {
        match self.status {
            RegionalGrantEstimateStatus::Draft => {
                return next == RegionalGrantEstimateStatus::Closed;
            }
            RegionalGrantEstimateStatus::Pending => {
                match next {
                    RegionalGrantEstimateStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalGrantEstimateStatus::Active => {
                match next {
                    RegionalGrantEstimateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalGrantEstimateStatus::Suspended => {
                return next == RegionalGrantEstimateStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalGrantEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional grant estimate {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional grant estimate", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalGrantEstimate {
        RegionalGrantEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional grant estimate in status draft.
pub fn new_regional_grant_estimate(code: &str, label: &str) -> RegionalGrantEstimate {
    RegionalGrantEstimate {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        created_at: 0,
        updated_at: 0,
        name: "".to_string(),
        timezone: "".to_string(),
        tags: Vec::new(),
        status: RegionalGrantEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_grant_estimate("code-884", "label-333");
        assert_eq!(item.status, RegionalGrantEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_regional_grant_estimate("code-884", "label-333");
        if item.move_to(RegionalGrantEstimateStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalGrantEstimateStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_grant_estimate("code-884", "label-333");
        item.status = RegionalGrantEstimateStatus::Closed;
        assert!(
            item.move_to(RegionalGrantEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
