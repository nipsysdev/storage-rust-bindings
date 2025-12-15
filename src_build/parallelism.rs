pub fn get_parallel_jobs() -> String {
    let num_jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let jobs = std::cmp::max(4, num_jobs.saturating_sub(4));

    println!(
        "cargo:warning=Using {} parallel jobs for build (available cores: {})",
        jobs, num_jobs
    );
    jobs.to_string()
}
