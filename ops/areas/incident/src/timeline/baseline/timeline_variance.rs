use super::*;

use acme_kit::text;

/// Where a timeline variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TimelineVarianceStatus {
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

impl std::fmt::Display for TimelineVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline variance as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineVariance {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub ceiling: f64,
    pub updated_at: i64,
    pub approved: bool,
    pub port: i64,
    pub description: String,
    pub status: TimelineVarianceStatus,
}

impl TimelineVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineVarianceStatus) -> bool {
        match self.status {
            TimelineVarianceStatus::Draft => {
                return next == TimelineVarianceStatus::Closed;
            }
            TimelineVarianceStatus::Pending => {
                match next {
                    TimelineVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineVarianceStatus::Active => {
                match next {
                    TimelineVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineVarianceStatus::Suspended => {
                match next {
                    TimelineVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: TimelineVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        (self.margin * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineVariance {
        TimelineVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            ceiling: self.ceiling,
            updated_at: self.updated_at,
            approved: self.approved,
            port: self.port,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline variance in status draft.
pub fn new_timeline_variance(code: &str, name: &str) -> TimelineVariance {
    TimelineVariance {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        ceiling: 0.0,
        updated_at: 0,
        approved: false,
        port: 0,
        description: "".to_string(),
        status: TimelineVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_timeline_variance("code-189", "name-561");
        assert_eq!(item.status, TimelineVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_timeline_variance("code-189", "name-561");
        if item.move_to(TimelineVarianceStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineVarianceStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_timeline_variance("code-189", "name-561");
        item.status = TimelineVarianceStatus::Closed;
        assert!(
            item.move_to(TimelineVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
