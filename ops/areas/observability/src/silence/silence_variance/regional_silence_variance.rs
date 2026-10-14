use super::*;

use acme_kit::text;

/// Where a regional silence variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSilenceVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalSilenceVarianceStatus {
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

impl std::fmt::Display for RegionalSilenceVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional silence variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceVariance {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub namespace: String,
    pub created_at: i64,
    pub quantity: i64,
    pub reviewer: String,
    pub ends_at: i64,
    pub floor_value: f64,
    pub unit: String,
    pub status: RegionalSilenceVarianceStatus,
}

impl RegionalSilenceVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSilenceVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSilenceVarianceStatus) -> bool {
        match self.status {
            RegionalSilenceVarianceStatus::Open => {
                match next {
                    RegionalSilenceVarianceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSilenceVarianceStatus::InReview => {
                return next == RegionalSilenceVarianceStatus::Resolved
                    || next == RegionalSilenceVarianceStatus::Archived;
            }
            RegionalSilenceVarianceStatus::Resolved => {
                return next == RegionalSilenceVarianceStatus::Reopened;
            }
            RegionalSilenceVarianceStatus::Reopened => {
                match next {
                    RegionalSilenceVarianceStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalSilenceVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional silence variance {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.floor_value + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSilenceVariance {
        RegionalSilenceVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            namespace: self.namespace.clone(),
            created_at: self.created_at,
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional silence variance in status open.
pub fn new_regional_silence_variance(code: &str, label: &str) -> RegionalSilenceVariance {
    RegionalSilenceVariance {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        margin: 0.0,
        namespace: "".to_string(),
        created_at: 0,
        quantity: 0,
        reviewer: "".to_string(),
        ends_at: 0,
        floor_value: 0.0,
        unit: "".to_string(),
        status: RegionalSilenceVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_silence_variance("code-724", "label-363");
        assert_eq!(item.status, RegionalSilenceVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_silence_variance("code-724", "label-363");
        if item.move_to(RegionalSilenceVarianceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSilenceVarianceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_silence_variance("code-724", "label-363");
        item.status = RegionalSilenceVarianceStatus::Archived;
        assert!(
            item.move_to(RegionalSilenceVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
