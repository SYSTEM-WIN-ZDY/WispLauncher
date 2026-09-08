//! Logical stream budgets for shared native HTTP/2 connections.
//!
//! HTTP/2 streams share one physical connection per authority, so they must
//! not consume the HTTP/1 connection permits held by `native_budget`.

use crate::util::fetch::{DownloadRoute, ProxyPolicy};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore};

const MAX_H2_STREAMS: usize = 128;
const MAX_H2_STREAMS_PER_AUTHORITY: usize = 32;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct AuthorityKey {
    authority: String,
    proxy: ProxyPolicy,
}

static AUTHORITY_BUDGETS: LazyLock<
    Mutex<HashMap<AuthorityKey, Arc<Semaphore>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));
static GLOBAL_BUDGET: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(MAX_H2_STREAMS)));

pub(crate) struct H2StreamPermit {
    _global: OwnedSemaphorePermit,
    _authority: Option<OwnedSemaphorePermit>,
}

fn budget(route: &DownloadRoute) -> Option<Arc<Semaphore>> {
    let authority = crate::util::fetch::url_authority(&route.url)?;
    let key = AuthorityKey {
        authority,
        proxy: route.proxy,
    };
    let mut budgets = AUTHORITY_BUDGETS.lock();
    if budgets.len() >= 256 {
        budgets.retain(|_, budget| Arc::strong_count(budget) > 1);
    }
    Some(
        budgets
            .entry(key)
            .or_insert_with(|| {
                Arc::new(Semaphore::new(MAX_H2_STREAMS_PER_AUTHORITY))
            })
            .clone(),
    )
}

pub(crate) async fn acquire(
    route: &DownloadRoute,
) -> Result<H2StreamPermit, AcquireError> {
    let authority = match budget(route) {
        Some(budget) => Some(budget.acquire_owned().await?),
        None => None,
    };
    let global = Arc::clone(&GLOBAL_BUDGET).acquire_owned().await?;
    Ok(H2StreamPermit {
        _global: global,
        _authority: authority,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::fetch::DownloadRouteSource;

    fn route() -> DownloadRoute {
        DownloadRoute {
            url: "https://h2-budget.example/file".to_string(),
            source: DownloadRouteSource::Official,
            is_mirror: false,
            allow_sensitive_headers: true,
            supports_range: true,
            proxy: ProxyPolicy::Direct,
        }
    }

    #[tokio::test]
    async fn authority_stream_budget_is_independent_from_connection_budget() {
        let route = route();
        let mut permits = Vec::new();
        for _ in 0..MAX_H2_STREAMS_PER_AUTHORITY {
            permits.push(acquire(&route).await.unwrap());
        }
        assert_eq!(budget(&route).unwrap().available_permits(), 0,);
        drop(permits);
        assert_eq!(
            budget(&route).unwrap().available_permits(),
            MAX_H2_STREAMS_PER_AUTHORITY,
        );
    }
}
