use core::panic;
use std::{
    arch::x86_64::_rdtsc,
    time::{Duration, Instant},
};

pub fn read_timer_cpu() -> u64 {
    unsafe { _rdtsc() }
}

pub fn estimate_cpu_frequency(millis_to_wait: u64) -> f64 {
    let os_start = Instant::now();
    let cpu_start = read_timer_cpu();

    let mut os_end;
    let mut os_elapsed = Duration::default();
    let wait_time = Duration::from_millis(millis_to_wait);
    while wait_time > os_elapsed {
        os_end = Instant::now();
        os_elapsed = os_end.saturating_duration_since(os_start);
    }

    let cpu_end = read_timer_cpu();
    let cpu_elapsed = cpu_end - cpu_start;
    let cpu_frequency = cpu_elapsed as f64 / os_elapsed.as_secs_f64() / 1_000.;

    return cpu_frequency;
}

#[macro_export]
macro_rules! time_function {
    ($id:literal) => {
        let label = {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let name = &name[..name.len() - 3];
            let start = match name.rfind(':') {
                Some(i) => i + 1,
                None => 0,
            };
            &name[start..]
        };
        let _drop_timer = DropTimer::new::<$id>(label);
    };
}

#[macro_export]
macro_rules! time_block {
    ($label:ident, $id:literal) => {
        let $label = DropTimer::new::<$id>($label);
    };
}

pub struct DropTimer {
    tsc_start: u64,
    label: &'static str,
    id: usize,
}
impl DropTimer {
    pub fn new<const ID: usize>(label: &'static str) -> DropTimer {
        if ID > MAX_ANCHORS {
            panic!("error");
        }

        let tsc_start = read_timer_cpu();
        Self {
            tsc_start,
            label,
            id: ID,
        }
    }
}
impl Drop for DropTimer {
    fn drop(&mut self) {
        let tsc_end = read_timer_cpu();
        let anchor = unsafe { PROFILER.anchors.get_unchecked_mut(self.id) };

        if anchor.label.is_empty() {
            anchor.label = self.label;
        };
        assert_eq!(
            anchor.label, self.label,
            "anchor id {} is reused with different labels ({} {})",
            self.id, self.label, anchor.label
        );

        anchor.tsc_elapsed += tsc_end - self.tsc_start;
    }
}

const MAX_ANCHORS: usize = 128;
static mut PROFILER: Profiler = Profiler::new();
struct Profiler {
    tsc_start: u64,
    tsc_end: u64,
    anchors: [Anchor; MAX_ANCHORS],
}
impl Profiler {
    const fn new() -> Profiler {
        Self {
            tsc_start: 0,
            tsc_end: 0,
            anchors: [Anchor::blank(); MAX_ANCHORS],
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Anchor {
    label: &'static str,
    tsc_elapsed: u64,
}

impl Anchor {
    const fn blank() -> Anchor {
        Self {
            label: "",
            tsc_elapsed: 0,
        }
    }
}

pub fn begin_profile() {
    let tsc_start = read_timer_cpu();
    unsafe { PROFILER.tsc_start = tsc_start };
}
pub fn end_profile_and_print() {
    let tsc_end = read_timer_cpu();
    unsafe { PROFILER.tsc_end = tsc_end };

    let tsc_total = unsafe { PROFILER.tsc_end - PROFILER.tsc_start };
    let millis_total = tsc_total as f64 / estimate_cpu_frequency(100);
    println!("Total time: {millis_total:.4}ms ({tsc_total})");

    let tsc_one_percent = tsc_total as f64 / 100.;
    let anchors = unsafe { PROFILER.anchors };
    for Anchor { label, tsc_elapsed } in anchors {
        if label.is_empty() {
            continue;
        };
        let perc_elapsed = tsc_elapsed as f64 / tsc_one_percent;
        println!("{label}: {tsc_elapsed} ({perc_elapsed:.2}%)");
    }
}
