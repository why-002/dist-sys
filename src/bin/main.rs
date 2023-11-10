use core::time;
use std::{thread::{self, sleep}, sync::{Mutex, Arc}};

fn main(){
    let a = Arc::new(Mutex::new(false));
    let c = Arc::clone(&a);
    let handle = thread::spawn( move || {
        loop {
            println!("Fun Loop");
            sleep(time::Duration::from_millis(1));
            let f = c.lock().unwrap();
            if *f {
                break;
            }
        }
    });

    for i in 0..100{
        println!("{}", i);
        sleep(time::Duration::from_millis(1));
    }
    let mut f = a.lock().unwrap();
    *f = true;
    let _ = handle.join();
}