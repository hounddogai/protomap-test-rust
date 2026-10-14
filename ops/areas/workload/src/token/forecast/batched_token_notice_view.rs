use super::*;

/// How screens and exports show a batched token notice.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenNoticeView {
    pub code: String,
    pub name_text: String,
    pub limit_text: String,
    pub category: String,
    pub sequence_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_batched_token_notice_view(item: &BatchedTokenNotice) -> BatchedTokenNoticeView {
    BatchedTokenNoticeView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        limit_text: item.limit.to_string(),
        category: item.category.clone(),
        sequence_text: item.sequence.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_token_notice_views(items: &[BatchedTokenNotice]) -> Vec<BatchedTokenNoticeView> {
    let mut views: Vec<BatchedTokenNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_token_notice_view(item));
    }
    views
}

pub fn sum_batched_token_notice_view(view: &BatchedTokenNoticeView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_sequence as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_token_notice("code-534", "name-9");
        let view = to_batched_token_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_token_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
