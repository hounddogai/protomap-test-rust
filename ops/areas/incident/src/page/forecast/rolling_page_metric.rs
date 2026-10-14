use super::*;

use acme_kit::text;

/// Where a rolling page metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPageMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingPageMetricStatus {
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

impl std::fmt::Display for RollingPageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling page metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetric {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub cluster: String,
    pub budget: f64,
    pub status: RollingPageMetricStatus,
}

impl RollingPageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPageMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPageMetricStatus) -> bool {
        match self.status {
            RollingPageMetricStatus::Draft => {
                return next == RollingPageMetricStatus::Suspended;
            }
            RollingPageMetricStatus::Pending => {
                return next == RollingPageMetricStatus::Active || next == RollingPageMetricStatus::Suspended;
            }
            RollingPageMetricStatus::Active => {
                return next == RollingPageMetricStatus::Suspended;
            }
            RollingPageMetricStatus::Suspended => {
                return next == RollingPageMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling page metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> RollingPageMetric {
        RollingPageMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling page metric in status draft.
pub fn new_rolling_page_metric(code: &str, title: &str) -> RollingPageMetric {
    RollingPageMetric {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        cluster: "".to_string(),
        budget: 0.0,
        status: RollingPageMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_page_metric("code-871", "title-81");
        assert_eq!(item.status, RollingPageMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_page_metric("code-871", "title-81");
        if item.move_to(RollingPageMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPageMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_page_metric("code-871", "title-81");
        item.status = RollingPageMetricStatus::Closed;
        assert!(
            item.move_to(RollingPageMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
