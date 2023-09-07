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
