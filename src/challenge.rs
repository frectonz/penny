use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// In-memory store for active ACME HTTP-01 challenges.
/// Maps challenge token to key authorization.
pub type ChallengeStore = Arc<RwLock<HashMap<String, String>>>;

/// Creates a new empty challenge store.
pub fn create_challenge_store() -> ChallengeStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Adds a challenge to the store.
pub async fn add_challenge(store: &ChallengeStore, token: String, key_auth: String) {
    store.write().await.insert(token, key_auth);
}

/// Gets a challenge's key authorization by token.
pub async fn get_challenge(store: &ChallengeStore, token: &str) -> Option<String> {
    store.read().await.get(token).cloned()
}

/// Removes a challenge from the store.
pub async fn remove_challenge(store: &ChallengeStore, token: &str) {
    store.write().await.remove(token);
}
