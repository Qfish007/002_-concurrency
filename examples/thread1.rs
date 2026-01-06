/*
   cargo run --example thread1
*/

use anyhow::{anyhow, Result};
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    id: usize,
    value: usize,
}

impl Msg {
    fn new(id: usize, value: usize) -> Self {
        Msg { id, value }
    }
}

fn main() -> Result<()> {
    // mpsc multi-producer, single-consumer channel
    let (tx, rx) = mpsc::channel();

    // multi-producer
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }
    drop(tx);

    // single-consumer channel
    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
        println!("Consumer exiting");
        42
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow!("thread join error:{:?}", e))?;

    println!("Secret: {:?}", secret);

    Ok(())
}

fn producer(id: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(id, value))?;
        let sleep_time = (rand::random::<u8>() as u64) * 10;
        thread::sleep(Duration::from_millis(sleep_time));

        if rand::random::<u8>().is_multiple_of(5) {
            println!("Producer {} exiting", id);
            break;
        }
    }

    Ok(())
}
