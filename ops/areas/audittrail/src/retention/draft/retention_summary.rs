use super::*;

use acme_kit::text;

/// Where a retention summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention summary as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSummary {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub grace_days: i64,
    pub ceiling: f64,
    pub host: String,
    pub segment: String,
    pub maximum: f64,
    pub rotation_days: i64,
    pub issuer: String,
    pub status: RetentionSummaryStatus,
}

impl RetentionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionSummaryStatus) -> bool {
        match self.status {
            RetentionSummaryStatus::Requested => {
                match next {
                    RetentionSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSummaryStatus::Granted => {
                match next {
                    RetentionSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSummaryStatus::Revoked => {
                match next {
                    RetentionSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: RetentionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.maximum * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionSummary {
        RetentionSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            host: self.host.clone(),
            segment: self.segment.clone(),
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a retention summary in status requested.
pub fn new_retention_summary(code: &str, title: &str) -> RetentionSummary {
    RetentionSummary {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        grace_days: 0,
        ceiling: 0.0,
        host: "".to_string(),
        segment: "".to_string(),
        maximum: 0.0,
        rotation_days: 0,
        issuer: "".to_string(),
        status: RetentionSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_retention_summary("code-443", "title-161");
        assert_eq!(item.status, RetentionSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_retention_summary("code-443", "title-161");
        if item.move_to(RetentionSummaryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionSummaryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_retention_summary("code-443", "title-161");
        item.status = RetentionSummaryStatus::Expired;
        assert!(
            item.move_to(RetentionSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
