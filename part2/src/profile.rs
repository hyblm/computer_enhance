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

static mut PROFILER: internals::Profiler = internals::Profiler::new();
pub fn begin_profile() {
    let tsc_start = read_timer_cpu();
    unsafe { PROFILER.tsc_start = tsc_start };
}
pub use internals::{end_profile_and_print, DropTimer};

#[cfg(not(feature = "profile"))]
mod internals {
    use super::{estimate_cpu_frequency, read_timer_cpu, PROFILER};

    #[macro_export]
    macro_rules! time_function {
        ($id:literal) => {};
    }

    #[macro_export]
    macro_rules! time_block {
        ($label:ident, $id:literal) => {};
    }

    pub fn end_profile_and_print() {
        let tsc_end = read_timer_cpu();
        unsafe { PROFILER.tsc_end = tsc_end };

        let tsc_total = unsafe { PROFILER.tsc_end - PROFILER.tsc_start };
        let millis_total = tsc_total as f64 / estimate_cpu_frequency(100);
        println!("Total time: {millis_total:.4}ms ({tsc_total})");
    }

    pub struct Profiler {
        pub tsc_start: u64,
        pub tsc_end: u64,
    }
    impl Profiler {
        pub const fn new() -> Profiler {
            Self {
                tsc_start: 0,
                tsc_end: 0,
            }
        }
    }
    pub struct DropTimer {}
}

#[cfg(feature = "profile")]
mod internals {
    use super::read_timer_cpu;
    use crate::profile::{estimate_cpu_frequency, PROFILER};

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

    pub fn end_profile_and_print() {
        let tsc_end = read_timer_cpu();
        unsafe { PROFILER.tsc_end = tsc_end };

        let tsc_total = unsafe { PROFILER.tsc_end - PROFILER.tsc_start };
        let millis_total = tsc_total as f64 / estimate_cpu_frequency(100);
        println!("Total time: {millis_total:.4}ms ({tsc_total})");

        let tsc_one_percent = tsc_total as f64 / 100.;
        let anchors = unsafe { PROFILER.anchors };
        for &Anchor {
            label,
            hit_count,
            tsc_elapsed_exclusive,
            tsc_elapsed_inclusive,
        } in anchors.iter().skip(1)
        {
            if label.is_empty() {
                continue;
            };
            let perc_elapsed_exclusive = tsc_elapsed_exclusive as f64 / tsc_one_percent;
            let perc_elapsed_inclusive = tsc_elapsed_inclusive as f64 / tsc_one_percent;
            println!("[{hit_count}] {label}: {tsc_elapsed_exclusive} ({perc_elapsed_exclusive:.2}%)  {tsc_elapsed_inclusive} ({perc_elapsed_inclusive:.2}%)");
        }
    }

    pub struct DropTimer {
        tsc_start: u64,
        label: &'static str,
        parent_id: usize,
        id: usize,
        old_inclusive: u64,
    }
    impl DropTimer {
        pub fn new<const ID: usize>(label: &'static str) -> DropTimer {
            if ID > MAX_ANCHORS {
                panic!("error");
            }

            let parent_id = unsafe { PROFILER.current_anchor };
            let anchor = unsafe { PROFILER.anchors.get_unchecked(ID) };
            let old_inclusive = anchor.tsc_elapsed_inclusive;

            unsafe { PROFILER.current_anchor = ID };
            let tsc_start = read_timer_cpu();

            Self {
                old_inclusive,
                tsc_start,
                label,
                id: ID,
                parent_id,
            }
        }
    }
    impl Drop for DropTimer {
        fn drop(&mut self) {
            let &mut Self {
                tsc_start,
                label,
                parent_id,
                id,
                old_inclusive,
            } = self;

            let tsc_elapsed = read_timer_cpu() - tsc_start;
            unsafe { PROFILER.current_anchor = parent_id };

            let anchor = unsafe { PROFILER.anchors.get_unchecked_mut(id) };
            let parent = unsafe { PROFILER.anchors.get_unchecked_mut(parent_id) };

            parent.tsc_elapsed_exclusive = parent.tsc_elapsed_exclusive.wrapping_sub(tsc_elapsed);

            anchor.tsc_elapsed_exclusive += tsc_elapsed;
            anchor.tsc_elapsed_inclusive = old_inclusive + tsc_elapsed;
            anchor.hit_count += 1;

            if anchor.label.is_empty() {
                anchor.label = label;
            };
            assert_eq!(
                anchor.label, label,
                "anchor id {id} is reused with different labels ({label} {})",
                anchor.label
            );
        }
    }

    const MAX_ANCHORS: usize = 128;
    pub struct Profiler {
        pub tsc_start: u64,
        pub tsc_end: u64,
        current_anchor: usize,
        anchors: [Anchor; MAX_ANCHORS],
    }
    impl Profiler {
        pub const fn new() -> Profiler {
            Self {
                tsc_start: 0,
                tsc_end: 0,
                current_anchor: 0,
                anchors: [Anchor::blank(); MAX_ANCHORS],
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct Anchor {
        label: &'static str,
        hit_count: u64,
        tsc_elapsed_exclusive: u64,
        tsc_elapsed_inclusive: u64,
    }

    impl Anchor {
        const fn blank() -> Anchor {
            Self {
                label: "",
                hit_count: 0,
                tsc_elapsed_exclusive: 0,
                tsc_elapsed_inclusive: 0,
            }
        }
    }
}
