use std::{time::Duration, arch::asm};

pub enum TickCounterFrequencyBase {
    Hardware,

    Measured(Duration)
}

pub struct TickCounter {
    start: u64
}

impl TickCounter {
    pub fn current() -> Self {
        TickCounter { start: start() }
    }

    pub fn elapsed(&self) -> u64 {
        stop() - self.start
    }
}

#[cfg(target_arch = "x86_64")]
pub fn frequency() -> (u64, TickCounterFrequencyBase) {
    let measure_duration = Duration::from_secs(1);
    let frequency_base = TickCounterFrequencyBase::Measured(measure_duration);
    (x86_64_measure_frequency(&measure_duration), frequency_base)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn x86_64_tick_counter() -> u64 {
    let mut reg_eax: u32;
    let mut reg_edx: u32;

    unsafe {
        asm!("rdtsc", out("eax") reg_eax, out("edx") reg_edx);
    }

    (reg_edx as u64) << 32 | (reg_eax as u64)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn x86_64_processor_id() -> (u64, u32) {
    let mut reg_eax: u32;
    let mut reg_edx: u32;
    let mut reg_ecx: u32;

    unsafe {
        asm!("rdtscp", out("eax") reg_eax, out("edx") reg_edx, out("ecx") reg_ecx);
    }

    ((reg_edx as u64) << 32 | (reg_eax as u64), reg_ecx)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn start() -> u64 {
    let rax: u64;

    unsafe {
        asm!(
            "mfence",
            "lfence",
            "rdtsc",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") rax
        );
    }
    rax
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn stop() -> u64 {
    let rax: u64;

    unsafe {
        asm!(
            "rdtsc",
            "lfence",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") rax
        );
    }
    rax
}

#[cfg(target_arch = "x86_64")]
pub fn x86_64_measure_frequency(measure_duration: &Duration) -> u64 {
    use std::thread;
    let counter_start = start();
    thread::sleep(*measure_duration);
    let counter_stop = stop();
    (((counter_stop - counter_start) as f64) / measure_duration.as_secs_f64()) as u64
}

pub fn precision_nanoseconds(frequency: u64) -> f64 {
    1.0e9_f64 / (frequency as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usage() {
        use std::{thread, time};
        let start = start();
        thread::sleep(time::Duration::from_millis(20));
        let elapsed_ticks = stop() - start;
        assert!(elapsed_ticks > 0);
    }

    #[test]
    fn basic_usage_with_helper() {
        use std::{thread, time};
        let tick_counter = TickCounter::current();
        thread::sleep(time::Duration::from_millis(20));
        let elapsed_ticks = tick_counter.elapsed();
        assert!(elapsed_ticks > 0);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_x86_64_counters() {
        use core::arch::x86_64::__rdtscp;
        use core::arch::x86_64::_rdtsc;

        let counter1 = x86_64_tick_counter();
        let counter2 = x86_64_tick_counter();
        let diff_tick_counter = counter2 - counter1;
        assert!(counter1 < counter2);
        assert!(diff_tick_counter > 0);

        let counter_start = start();
        let counter_stop = stop();
        let diff_tick_counter2 = counter_stop - counter_start;
        assert!(counter_start < counter_stop);
        assert!(diff_tick_counter2 > 0);

        let counter3 = unsafe { _rdtsc() };
        let counter4 = unsafe { _rdtsc() };
        let diff_tick_counter3 = counter4 - counter3;
        assert!(counter3 < counter4);
        assert!(diff_tick_counter3 > 0);

        let mut ecx: u32 = 0;
        let ptr_ecx: *mut u32 = (&mut ecx) as *mut u32;
        let counter5 = unsafe { __rdtscp(ptr_ecx) };
        let cpu_core_id_1 = ecx;

        let counter6 = unsafe { __rdtscp(ptr_ecx) };
        let cpu_core_id_2 = ecx;
        let diff_tick_rdtscp = counter6 - counter5;

        assert!(counter5 < counter6);
        assert!(diff_tick_rdtscp > 0);
        assert!(cpu_core_id_1 == cpu_core_id_2);

        let (counter7, cpu_core_id_3) = x86_64_processor_id();
        let (counter8, cpu_core_id_4) = x86_64_processor_id();
        let diff_tick_asm_rdtscp = counter8 - counter7;

        assert!(counter7 < counter8);
        assert!(cpu_core_id_1 == cpu_core_id_3);
        assert!(cpu_core_id_3 == cpu_core_id_4);
        assert!(diff_tick_asm_rdtscp > 0);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_x86_64_counter_frequency() {
        let (counter_frequency, frequency_base_) = frequency();
        assert!(counter_frequency > 0);
        let estimated_duration = match frequency_base_ {
            TickCounterFrequencyBase::Hardware => None,
            TickCounterFrequencyBase::Measured(duration) => Some(duration)
        };
        assert_eq!(estimated_duration, Some(Duration::from_millis(1000)));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_x86_64_counter_accuracy() {
        let counter_frequency = 24_000_000;
        let counter_accuracy = precision_nanoseconds(counter_frequency);
        assert_eq!((counter_accuracy as u64), 41);
    }
}

