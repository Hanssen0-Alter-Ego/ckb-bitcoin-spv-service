//! Read-only views of the observations made by the synchronization loop.

use std::{
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

#[derive(Clone, Default)]
pub(crate) struct SyncStatusCache(Arc<RwLock<SyncStatus>>);

#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    bitcoin_tip: Option<ObservedTip>,
    local_header_tip: Option<ObservedTip>,
    confirmed_client: Option<ObservedClient>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservedTip {
    height: u32,
    observed_at: Option<u64>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservedClient {
    start_height: u32,
    end_height: u32,
    observed_at: Option<u64>,
}

fn now() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

impl SyncStatusCache {
    pub(crate) fn snapshot(&self) -> SyncStatus {
        *self.0.read().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn observe_bitcoin_tip(&self, height: u32) {
        self.0
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .bitcoin_tip = Some(ObservedTip {
            height,
            observed_at: now(),
        });
    }

    pub(crate) fn observe_local_tip(&self, height: u32) {
        self.0
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .local_header_tip = Some(ObservedTip {
            height,
            observed_at: now(),
        });
    }

    pub(crate) fn observe_confirmed_client(&self, start_height: u32, end_height: u32) {
        self.0
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .confirmed_client = Some(ObservedClient {
            start_height,
            end_height,
            observed_at: now(),
        });
    }
}
