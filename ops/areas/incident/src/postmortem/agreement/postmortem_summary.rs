use super::*;

use acme_kit::text;

/// Where a postmortem summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PostmortemSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PostmortemSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSummary {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub country: String,
    pub channel: String,
    pub notes: String,
    pub minimum: f64,
    pub cost: f64,
    pub scope: String,
    pub status: PostmortemSummaryStatus,
}

impl PostmortemSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemSummaryStatus) -> bool {
        match self.status {
            PostmortemSummaryStatus::Proposed => {
                return next == PostmortemSummaryStatus::Approved || next == PostmortemSummaryStatus::Rejected;
            }
            PostmortemSummaryStatus::Approved => {
                match next {
                    PostmortemSummaryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemSummaryStatus::Rejected => {
                match next {
                    PostmortemSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PostmortemSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem summary", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.cost - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemSummary {
        PostmortemSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            country: self.country.clone(),
            channel: self.channel.clone(),
            notes: self.notes.clone(),
            minimum: self.minimum,
            cost: self.cost,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a postmortem summary in status proposed.
pub fn new_postmortem_summary(code: &str, label: &str) -> PostmortemSummary {
    PostmortemSummary {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        country: "".to_string(),
        channel: "".to_string(),
        notes: "".to_string(),
        minimum: 0.0,
        cost: 0.0,
        scope: "".to_string(),
        status: PostmortemSummaryStatus::Proposed,
    }
}
