use std::{
  marker::PhantomData,
  sync::{Arc, Condvar, Mutex},
  time::Duration,
};

use rand::Rng;

struct Inner {
  /// The number of threads waiting for the group to be done.
  waiter_count: Mutex<usize>,
  /// Used to block the thread and wait for the other threads in the group to be done.
  cond_var: Condvar,
}

struct Guard;
struct NotGuard;

struct WaitGroup<T> {
  inner: Arc<Inner>,
  _p: PhantomData<T>,
}

impl WaitGroup<NotGuard> {
  pub fn new() -> WaitGroup<NotGuard> {
    WaitGroup {
      inner: Arc::new(Inner {
        waiter_count: Mutex::new(0),
        cond_var: Condvar::new(),
      }),
      _p: PhantomData,
    }
  }
}

impl WaitGroup<NotGuard> {
  pub fn add(&self) -> WaitGroup<Guard> {
    let mut waiter_count = self.inner.waiter_count.lock().unwrap();

    *waiter_count += 1;

    WaitGroup {
      inner: Arc::clone(&self.inner),
      _p: PhantomData,
    }
  }

  pub fn wait(&self) {
    let guard = self.inner.waiter_count.lock().unwrap();

    let _waiter_count = self
      .inner
      .cond_var
      .wait_while(guard, |waiter_count| *waiter_count > 0)
      .unwrap();
  }
}

impl<T> Drop for WaitGroup<T> {
  fn drop(&mut self) {
    let mut waiter_count = self.inner.waiter_count.lock().unwrap();
    println!(
      "WaitGroup::Drop. thread_id={:?}, waiter_count={}",
      std::thread::current().id(),
      *waiter_count
    );
    if *waiter_count > 0 {
      *waiter_count -= 1;
    }

    self.inner.cond_var.notify_all();
  }
}

fn main() {
  let wg = WaitGroup::new();

  for _ in 0..=5 {
    let wg = wg.add();

    let _ = std::thread::spawn(move || {
      let secs = Duration::from_secs(rand::thread_rng().gen_range(1..=5));
      println!(
        "thread_id={:?} SLEEPING for {:?}",
        std::thread::current().id(),
        secs
      );
      std::thread::sleep(secs);
      drop(wg);
    });
  }

  println!("waiting for threads");
  wg.wait();
  println!("done");
}
