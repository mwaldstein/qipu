use crate::results::{Cache, CacheKey, ResultRecord};

pub fn compute_cache_key(
    scenario_yaml: &str,
    prompt: &str,
    prime_output: &str,
    tool: &str,
    model: &str,
    qipu_version: &str,
) -> CacheKey {
    CacheKey::compute(
        scenario_yaml,
        prompt,
        prime_output,
        tool,
        model,
        qipu_version,
    )
}

pub fn check_cache(cache: &Cache, cache_key: &CacheKey) -> anyhow::Result<Option<ResultRecord>> {
    Ok(cache.get(cache_key))
}
