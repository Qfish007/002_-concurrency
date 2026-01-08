use anyhow::{Ok, Result};
use concurrency::*;
use rand::Rng;
use std::thread;

const N: usize = 2;
const M: usize = 4;
fn main() -> Result<()> {
    let metrics = CmapMetrics::new(); // or CmapMetrics::new_with_capacity(1000)

    for idx in 0..N {
        task_worker(idx, metrics.clone())?;
    }

    for _ in 0..M {
        request_worker(metrics.clone())?;
    }

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        println!("Metrics snapshot: {:?}", metrics.snapshot());
    }

    #[allow(unreachable_code)]
    Ok(())
}

fn task_worker(idx: usize, metrics: CmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(std::time::Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.worker{}", idx))?;
        }

        #[allow(unreachable_code)]
        Ok(())
    });

    Ok(())
}

fn request_worker(metrics: CmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(std::time::Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(1..5);
            metrics.inc(format!("req.page{}", page))?;
        }

        #[allow(unreachable_code)]
        Ok(())
    });

    Ok(())
}
