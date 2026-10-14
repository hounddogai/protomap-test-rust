use super::*;

use acme_kit::text;

/// Where a postmortem metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PostmortemMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem metric as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMetric {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub ends_at: i64,
    pub cost: f64,
    pub revision: i64,
    pub weight: f64,
    pub status: PostmortemMetricStatus,
}

impl PostmortemMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemMetricStatus) -> bool {
        match self.status {
            PostmortemMetricStatus::New => {
                match next {
                    PostmortemMetricStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemMetricStatus::Queued => {
                return next == PostmortemMetricStatus::Done;
            }
            PostmortemMetricStatus::Running => {
                match next {
                    PostmortemMetricStatus::Done | PostmortemMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemMetricStatus::Done => {
                match next {
                    PostmortemMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: PostmortemMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.cost - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemMetric {
        PostmortemMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            ends_at: self.ends_at,
            cost: self.cost,
            revision: self.revision,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem metric in status new.
pub fn new_postmortem_metric(code: &str, title: &str) -> PostmortemMetric {
    PostmortemMetric {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        ends_at: 0,
        cost: 0.0,
        revision: 0,
        weight: 0.0,
        status: PostmortemMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_postmortem_metric("code-835", "title-963");
        assert_eq!(item.status, PostmortemMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_postmortem_metric("code-835", "title-963");
        if item.move_to(PostmortemMetricStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemMetricStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_postmortem_metric("code-835", "title-963");
        item.status = PostmortemMetricStatus::Failed;
        assert!(
            item.move_to(PostmortemMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
