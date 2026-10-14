use super::*;

use acme_kit::text;

/// Where a regional forecast bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalForecastBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalForecastBundleStatus {
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

impl std::fmt::Display for RegionalForecastBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional forecast bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundle {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub version: i64,
    pub reviewer: String,
    pub external_ref: String,
    pub phone: String,
    pub region: String,
    pub active: bool,
    pub status: RegionalForecastBundleStatus,
}

impl RegionalForecastBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalForecastBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalForecastBundleStatus) -> bool {
        match self.status {
            RegionalForecastBundleStatus::Open => {
                return next == RegionalForecastBundleStatus::InReview
                    || next == RegionalForecastBundleStatus::Reopened;
            }
            RegionalForecastBundleStatus::InReview => {
                match next {
                    RegionalForecastBundleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalForecastBundleStatus::Resolved => {
                return next == RegionalForecastBundleStatus::Reopened
                    || next == RegionalForecastBundleStatus::Archived;
            }
            RegionalForecastBundleStatus::Reopened => {
                match next {
                    RegionalForecastBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalForecastBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional forecast bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional forecast bundle", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalForecastBundle {
        RegionalForecastBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            version: self.version,
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            phone: self.phone.clone(),
            region: self.region.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional forecast bundle in status open.
pub fn new_regional_forecast_bundle(code: &str, label: &str) -> RegionalForecastBundle {
    RegionalForecastBundle {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        ttl_seconds: 0,
        version: 0,
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        phone: "".to_string(),
        region: "".to_string(),
        active: false,
        status: RegionalForecastBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_forecast_bundle("code-334", "label-677");
        assert_eq!(item.status, RegionalForecastBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_regional_forecast_bundle("code-334", "label-677");
        if item.move_to(RegionalForecastBundleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalForecastBundleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_forecast_bundle("code-334", "label-677");
        item.status = RegionalForecastBundleStatus::Archived;
        assert!(
            item.move_to(RegionalForecastBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
