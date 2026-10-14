use super::*;

use acme_kit::text;

/// Where a usage threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl UsageThresholdStatus {
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

impl std::fmt::Display for UsageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageThreshold {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub cluster: String,
    pub notes: String,
    pub tenant_id: String,
    pub country: String,
    pub margin: f64,
    pub checksum: String,
    pub status: UsageThresholdStatus,
}

impl UsageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageThresholdStatus) -> bool {
        match self.status {
            UsageThresholdStatus::Draft => {
                match next {
                    UsageThresholdStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageThresholdStatus::Pending => {
                match next {
                    UsageThresholdStatus::Active | UsageThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageThresholdStatus::Active => {
                match next {
                    UsageThresholdStatus::Suspended | UsageThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageThresholdStatus::Suspended => {
                return next == UsageThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.budget - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageThreshold {
        UsageThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            country: self.country.clone(),
            margin: self.margin,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a usage threshold in status draft.
pub fn new_usage_threshold(code: &str, title: &str) -> UsageThreshold {
    UsageThreshold {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        cluster: "".to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        country: "".to_string(),
        margin: 0.0,
        checksum: "".to_string(),
        status: UsageThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_usage_threshold("code-103", "title-531");
        assert_eq!(item.status, UsageThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_usage_threshold("code-103", "title-531");
        if item.move_to(UsageThresholdStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageThresholdStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_usage_threshold("code-103", "title-531");
        item.status = UsageThresholdStatus::Closed;
        assert!(
            item.move_to(UsageThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
