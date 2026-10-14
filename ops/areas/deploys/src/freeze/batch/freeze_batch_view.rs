use super::*;

/// How screens and exports show a freeze batch.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchView {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub retries_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_freeze_batch_view(item: &FreezeBatch) -> FreezeBatchView {
    FreezeBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        contact_email: item.contact_email.clone(),
        retries_text: item.retries.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_batch_views(items: &[FreezeBatch]) -> Vec<FreezeBatchView> {
    let mut views: Vec<FreezeBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_batch_view(item));
    }
    views
}

pub fn sum_freeze_batch_view(view: &FreezeBatchView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_batch("code-825", "name-792");
        let view = to_freeze_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
