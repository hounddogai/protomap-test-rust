use super::*;

use acme_kit::text;

/// Where a slo mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloMappingStatus {
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

impl std::fmt::Display for SloMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloMapping {
    pub code: String,
    pub title: String,
    pub region: String,
    pub window_days: i64,
    pub namespace: String,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub discount: f64,
    pub score: f64,
    pub scope: String,
    pub status: SloMappingStatus,
}

impl SloMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloMappingStatus) -> bool {
        match self.status {
            SloMappingStatus::Open => {
                match next {
                    SloMappingStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloMappingStatus::InReview => {
                match next {
                    SloMappingStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloMappingStatus::Resolved => {
                return next == SloMappingStatus::Reopened || next == SloMappingStatus::Archived;
            }
            SloMappingStatus::Reopened => {
                match next {
                    SloMappingStatus::Archived => {
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

    pub fn move_to(&mut self, next: SloMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloMapping {
        SloMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            discount: self.discount,
            score: self.score,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo mapping in status open.
pub fn new_slo_mapping(code: &str, title: &str) -> SloMapping {
    SloMapping {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        window_days: 0,
        namespace: "".to_string(),
        duration_minutes: 0,
        contact_email: "".to_string(),
        discount: 0.0,
        score: 0.0,
        scope: "".to_string(),
        status: SloMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_mapping("code-168", "title-636");
        assert_eq!(item.status, SloMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_slo_mapping("code-168", "title-636");
        if item.move_to(SloMappingStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloMappingStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_mapping("code-168", "title-636");
        item.status = SloMappingStatus::Archived;
        assert!(item.move_to(SloMappingStatus::Open).is_err(), "expected move to fail");
    }
}
