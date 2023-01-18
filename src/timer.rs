//! Countdown timers
//!
//! Configures the board's TimerB peripherals into periodic countdown timers. Each peripheral
//! consists of a main timer and multiple "sub-timers". Sub-timers have their own thresholds and
//! interrupts but share their countdowns with their main timer.
//!
//! This module also contains traits used by other HAL modules that depend on TimerB, such as
//! `Capture` and `Pwm`.

use crate::clock::{Aclk, Smclk};
use crate::gpio::*;
use crate::hw_traits::timera::{CCRn, Tassel, TimerA};
use core::marker::PhantomData;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use crate::pac;

pub use crate::hw_traits::timera::{
    TimerDiv, TimerExDiv, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};

// Trait effectively sealed by CCRn
/// Trait indicating that the peripheral can be used as a sub-timer, PWM, or capture
pub trait CapCmp<C>: CCRn<C> {}
impl<T: CCRn<C>, C> CapCmp<C> for T {}

// Trait effectively sealed by TimerB
/// Trait indicating that the peripheral can be used as a timer
pub trait TimerPeriph: TimerA + CapCmp<CCR0> {
    /// Pin type used for external TAxCLK of this timer
    type Taxclk;
}

// Traits effectively sealed by CCRn
/// Trait indicating that the peripheral has 3 capture compare registers
pub trait CapCmpTimer3: TimerPeriph + CapCmp<CCR1> + CapCmp<CCR2> {}
/// Trait indicating that the peripheral has 7 capture compare registers
pub trait CapCmpTimer7:
    TimerPeriph
    + CapCmp<CCR1>
    + CapCmp<CCR2>
    + CapCmp<CCR3>
    + CapCmp<CCR4>
    + CapCmp<CCR5>
    + CapCmp<CCR6>
{
}

// impl TimerPeriph for pac::TA0 {
//     type Taxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>;
// }
// impl CapCmpTimer3 for pac::TA0 {}

impl TimerPeriph for pac::TA0 {
    type Taxclk = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
}
impl CapCmpTimer3 for pac::TA0 {}

impl TimerPeriph for pac::TA1 {
    type Taxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
}
impl CapCmpTimer3 for pac::TA1 {}

impl TimerPeriph for pac::TA2 {
    type Taxclk = Pin<P3, Pin4, Alternate1<Input<Floating>>>;
}
impl CapCmpTimer3 for pac::TA2 {}

impl TimerPeriph for pac::TA3 {
    type Taxclk = Pin<P5, Pin4, Alternate2<Input<Floating>>>;
}
// impl CapCmpTimer7 for pac::TA3 {}
impl CapCmpTimer3 for pac::TA3 {}

/// Configuration object for the TimerB peripheral
///
/// Used to configure `Timer`, `Capture`, and `Pwm`, which all use the TimerB peripheral.
pub struct TimerConfig<T: TimerPeriph> {
    _timer: PhantomData<T>,
    sel: Tassel,
    div: TimerDiv,
    ex_div: TimerExDiv,
}

impl<T: TimerPeriph> TimerConfig<T> {
    /// Configure timer clock source to ACLK
    #[inline]
    pub fn aclk(_aclk: &Aclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tassel::Aclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to SMCLK
    #[inline]
    pub fn smclk(_smclk: &Smclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tassel::Smclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to TACLK
    #[inline]
    pub fn taclk(_pin: T::Taxclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tassel::Taxclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure the normal clock divider and expansion clock divider settings
    #[inline]
    pub fn clk_div(self, div: TimerDiv, ex_div: TimerExDiv) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: self.sel,
            div,
            ex_div,
        }
    }

    #[inline]
    pub(crate) fn write_regs(self, timer: &T) {
        timer.reset();
        timer.set_taidex(self.ex_div);
        timer.config_clock(self.sel, self.div);
    }
}

/// Main timer and sub-timers for timer peripherals with 3 capture-compare registers
pub struct TimerParts3<T: CapCmpTimer3> {
    /// Main timer
    pub timer: Timer<T>,
    /// Timer interrupt vector
    pub taxiv: TAxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register)
    pub subtimer2: SubTimer<T, CCR2>,
}

impl<T: CapCmpTimer3> TimerParts3<T> {
    /// Create new set of timers out of a TAx peripheral
    #[inline(always)]
    pub fn new(_timer: T, config: TimerConfig<T>) -> Self {
        config.write_regs(unsafe { &T::steal() });
        Self {
            timer: Timer::new(),
            taxiv: TAxIV(PhantomData),
            subtimer1: SubTimer::new(),
            subtimer2: SubTimer::new(),
        }
    }
}

/// Main timer and sub-timers for timer peripherals with 7 capture-compare registers
pub struct TimerParts7<T: CapCmpTimer7> {
    /// Main timer
    pub timer: Timer<T>,
    /// Timer interrupt vector
    pub taxiv: TAxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register)
    pub subtimer2: SubTimer<T, CCR2>,
    /// Sub-timer 3 (derived from CCR3 register)
    pub subtimer3: SubTimer<T, CCR3>,
    /// Sub-timer 4 (derived from CCR4 register)
    pub subtimer4: SubTimer<T, CCR4>,
    /// Sub-timer 5 (derived from CCR5 register)
    pub subtimer5: SubTimer<T, CCR5>,
    /// Sub-timer 6 (derived from CCR6 register)
    pub subtimer6: SubTimer<T, CCR6>,
}

impl<T: CapCmpTimer7> TimerParts7<T> {
    /// Create new set of timers out of a TAx peripheral
    #[inline(always)]
    pub fn new(_timer: T, config: TimerConfig<T>) -> Self {
        config.write_regs(unsafe { &T::steal() });
        Self {
            timer: Timer::new(),
            taxiv: TAxIV(PhantomData),
            subtimer1: SubTimer::new(),
            subtimer2: SubTimer::new(),
            subtimer3: SubTimer::new(),
            subtimer4: SubTimer::new(),
            subtimer5: SubTimer::new(),
            subtimer6: SubTimer::new(),
        }
    }
}

/// Main periodic countdown timer
pub struct Timer<T: TimerPeriph>(PhantomData<T>);

impl<T: TimerPeriph> Timer<T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

/// Sub-timer associated with a main timer
///
/// Each sub-timer has its own interrupt mechanism and threshold, but shares its countdown value
/// with its main timer.
pub struct SubTimer<T: CapCmp<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmp<C>, C> SubTimer<T, C> {
    fn new() -> Self {
        Self(PhantomData, PhantomData)
    }
}

/// Indicates which sub/main timer caused the interrupt to fire
pub enum TimerVector {
    /// No pending interrupt
    NoInterrupt,
    /// Interrupt caused by sub-timer 1
    SubTimer1,
    /// Interrupt caused by sub-timer 2
    SubTimer2,
    /// Interrupt caused by sub-timer 3
    SubTimer3,
    /// Interrupt caused by sub-timer 4
    SubTimer4,
    /// Interrupt caused by sub-timer 5
    SubTimer5,
    /// Interrupt caused by sub-timer 6
    SubTimer6,
    /// Interrupt caused by main timer overflow
    MainTimer,
}

#[inline]
pub(crate) fn read_taxiv<T: TimerA>(timer: &T) -> TimerVector {
    match timer.taxiv_rd() {
        0 => TimerVector::NoInterrupt,
        2 => TimerVector::SubTimer1,
        4 => TimerVector::SubTimer2,
        6 => TimerVector::SubTimer3,
        8 => TimerVector::SubTimer4,
        10 => TimerVector::SubTimer5,
        12 => TimerVector::SubTimer6,
        14 => TimerVector::MainTimer,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

/// Interrupt vector register for determining which timer caused an ISR
pub struct TAxIV<T>(PhantomData<T>);

impl<T: TimerA> TAxIV<T> {
    #[inline]
    /// Read the timer interrupt vector. Automatically resets corresponding interrupt flag.
    pub fn interrupt_vector(&mut self) -> TimerVector {
        let timer = unsafe { T::steal() };
        read_taxiv(&timer)
    }
}

impl<T: TimerPeriph + CapCmp<CCR0>> CountDown for Timer<T> {
    type Time = u16;

    #[inline]
    fn start<U: Into<Self::Time>>(&mut self, count: U) {
        let timer = unsafe { T::steal() };
        timer.stop();
        timer.set_ccrn(count.into());
        timer.upmode();
    }

    #[inline]
    fn wait(&mut self) -> nb::Result<(), void::Void> {
        let timer = unsafe { T::steal() };
        if timer.taifg_rd() {
            timer.taifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: TimerPeriph + CapCmp<CCR0>> Cancel for Timer<T> {
    type Error = void::Void;

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        let timer = unsafe { T::steal() };
        timer.stop();
        Ok(())
    }
}

impl<T: TimerPeriph> Periodic for Timer<T> {}

impl<T: TimerPeriph> Timer<T> {
    /// Enable timer countdown expiration interrupts
    #[inline(always)]
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.taie_set();
    }

    /// Disable timer countdown expiration interrupts
    #[inline(always)]
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.taie_clr();
    }
}

impl<T: CapCmp<C>, C> SubTimer<T, C> {
    #[inline]
    /// Set the threshold for one of the sub-timers. Once the main timer counts to this threshold
    /// the sub-timer will fire. Note that the main timer resets once it counts to its own
    /// threshold, not the sub-timer thresholds. It follows that the sub-timer threshold must be
    /// less than the main threshold for it to fire.
    pub fn set_count(&mut self, count: u16) {
        let timer = unsafe { T::steal() };
        timer.set_ccrn(count);
        timer.ccifg_clr();
    }

    #[inline]
    /// Wait for the sub-timer to fire
    pub fn wait(&mut self) -> nb::Result<(), void::Void> {
        let timer = unsafe { T::steal() };
        if timer.ccifg_rd() {
            timer.ccifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline(always)]
    /// Enable the sub-timer interrupts
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline(always)]
    /// Disable the sub-timer interrupts
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_clr();
    }
}
