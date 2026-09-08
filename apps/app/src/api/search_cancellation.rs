use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use tokio_util::sync::CancellationToken;

static SEARCH_CANCELLATIONS: OnceLock<
    Mutex<HashMap<String, Arc<CancellationToken>>>,
> = OnceLock::new();

fn cancellations() -> &'static Mutex<HashMap<String, Arc<CancellationToken>>> {
    SEARCH_CANCELLATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct SearchCancellation {
    request_id: String,
    token: Arc<CancellationToken>,
}

pub struct PendingCancellation {
    request_id: String,
    token: Arc<CancellationToken>,
}

impl PendingCancellation {
    pub fn expire(self) {
        let mut requests = cancellations()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let is_still_pending = requests
            .get(&self.request_id)
            .is_some_and(|token| Arc::ptr_eq(token, &self.token));

        if is_still_pending {
            requests.remove(&self.request_id);
        }
    }
}

impl SearchCancellation {
    pub fn register(request_id: String) -> Self {
        let mut requests = cancellations()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let token = match requests.remove(&request_id) {
            Some(token) if token.is_cancelled() => token,
            Some(token) => {
                token.cancel();
                Arc::new(CancellationToken::new())
            }
            None => Arc::new(CancellationToken::new()),
        };
        requests.insert(request_id.clone(), Arc::clone(&token));

        Self { request_id, token }
    }

    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }
}

impl Drop for SearchCancellation {
    fn drop(&mut self) {
        let mut requests = cancellations()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owns_registration = requests
            .get(&self.request_id)
            .is_some_and(|token| Arc::ptr_eq(token, &self.token));

        if owns_registration {
            requests.remove(&self.request_id);
        }
    }
}

pub fn cancel(request_id: &str) -> Option<PendingCancellation> {
    let mut requests = cancellations()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(token) = requests.get(request_id) {
        token.cancel();
        return None;
    }

    let token = Arc::new(CancellationToken::new());
    token.cancel();
    requests.insert(request_id.to_string(), Arc::clone(&token));
    Some(PendingCancellation {
        request_id: request_id.to_string(),
        token,
    })
}

#[cfg(test)]
mod tests {
    use super::{SearchCancellation, cancel, cancellations};

    #[test]
    fn cancellation_before_registration_is_preserved() {
        let request_id = "search-cancelled-before-registration";

        let _pending = cancel(request_id);
        let cancellation = SearchCancellation::register(request_id.to_string());

        assert!(cancellation.token.is_cancelled());
        drop(cancellation);

        assert!(
            !cancellations()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .contains_key(request_id)
        );
    }

    #[test]
    fn stale_guard_does_not_remove_a_new_registration() {
        let request_id = "search-registration-replacement";
        let stale = SearchCancellation::register(request_id.to_string());
        let current = SearchCancellation::register(request_id.to_string());

        drop(stale);
        let _ = cancel(request_id);

        assert!(current.token.is_cancelled());
        drop(current);
        assert!(
            !cancellations()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .contains_key(request_id)
        );
    }

    #[test]
    fn unused_pending_cancellation_can_expire() {
        let request_id = "search-cancellation-expiry";
        let pending =
            cancel(request_id).expect("a new cancellation should be pending");

        pending.expire();

        assert!(
            !cancellations()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .contains_key(request_id)
        );
    }
}
