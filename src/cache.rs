use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{ Duration, Instant };

/// A cache entry that stores a value and its expiration timestamp.
struct CacheEntry<T> {
    /// The stored value of generic type `T`.
    value: T,
    /// The `Instant` when this entry is considered expired and should no longer be returned by the cache.
    expires_at: Instant,
}

/// A thread-safe, generic cache for storing values associated with string keys.
/// Values in the cache have a default time-to-live (TTL) after which they are considered expired.
pub struct Cache<T> {
    /// A map from string keys to cache entries, wrapped in a Mutex for thread safety.
    entries: Mutex<HashMap<String, CacheEntry<T>>>,
    /// The default TTL for new cache entries.
    default_ttl: Duration,
}

impl<T> Cache<T> {
    /// Creates a new cache with the given default TTL for its entries.
    ///
    /// # Arguments
    ///
    /// * `default_ttl` - A `Duration` representing the default time-to-live for cache entries.
    pub fn new(default_ttl: Duration) -> Self {
        Cache {
            entries: Mutex::new(HashMap::new()),
            default_ttl,
        }
    }

    /// Retrieves a value from the cache by its key, if it exists and has not expired.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key of the cache entry to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<T>` which is `Some(T)` if the key exists and has not expired, or `None` if the key does not exist or has expired.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `cache` is an instance of `Cache<String>`.
    /// if let Some(value) = cache.get("my_key") {
    ///     println!("Found value: {}", value);
    /// } else {
    ///     println!("Value not found or expired.");
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<T> where T: Clone {
        let entries_lock = self.entries.lock().unwrap();
        entries_lock.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at { Some(entry.value.clone()) } else { None }
        })
    }

    /// Inserts a value into the cache with the specified key and the default TTL.
    ///
    /// # Arguments
    ///
    /// * `key` - A string representing the key under which to store the value.
    /// * `value` - The value to store in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `cache` is an instance of `Cache<String>`.
    /// cache.set("my_key".to_string(), "my_value".to_string());
    /// ```
    pub fn set(&self, key: String, value: T) {
        let mut entries_lock = self.entries.lock().unwrap();
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.default_ttl,
        };
        entries_lock.insert(key, entry);
    }
}
