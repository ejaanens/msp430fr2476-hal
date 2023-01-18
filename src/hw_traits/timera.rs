use super::Steal;
use crate::pac;

pub enum Tassel {
    Taxclk,
    Aclk,
    Smclk,
    Inclk,
}

/// Timer clock divider
pub enum TimerDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 4
    _4,
    /// Divide by 8
    _8,
}

/// Timer expansion clock divider, applied on top of the normal clock divider
pub enum TimerExDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 3
    _3,
    /// Divide by 4
    _4,
    /// Divide by 5
    _5,
    /// Divide by 6
    _6,
    /// Divide by 7
    _7,
    /// Divide by 8
    _8,
}

pub enum Outmod {
    Out,
    Set,
    ToggleReset,
    SetReset,
    Toggle,
    Reset,
    ToggleSet,
    ResetSet,
}

pub enum Cm {
    NoCap,
    RisingEdge,
    FallingEdge,
    BothEdges,
}

pub enum Ccis {
    InputA,
    InputB,
    Gnd,
    Vcc,
}

pub trait TimerA: Steal {
    /// Reset timer countdown
    fn reset(&self);

    /// Set to upmode, reset timer, and clear interrupts
    fn upmode(&self);
    /// Set to continuous mode, reset timer, and clear interrupts
    fn continuous(&self);

    /// Apply clock select settings
    fn config_clock(&self, tassel: Tassel, div: TimerDiv);

    /// Check if timer is stopped
    fn is_stopped(&self) -> bool;

    /// Stop timer
    fn stop(&self);

    /// Set expansion register clock divider settings
    fn set_taidex(&self, taidex: TimerExDiv);

    fn taifg_rd(&self) -> bool;
    fn taifg_clr(&self);

    fn taie_set(&self);
    fn taie_clr(&self);

    fn taxiv_rd(&self) -> u16;
}

pub trait CCRn<C>: Steal {
    fn set_ccrn(&self, count: u16);
    fn get_ccrn(&self) -> u16;

    fn config_outmod(&self, outmod: Outmod);
    fn config_cap_mode(&self, cm: Cm, ccis: Ccis);

    fn ccifg_rd(&self) -> bool;
    fn ccifg_clr(&self);

    fn ccie_set(&self);
    fn ccie_clr(&self);

    fn cov_ccifg_rd(&self) -> (bool, bool);
    fn cov_ccifg_clr(&self);
}

/// Label for capture-compare register 0
pub struct CCR0;
/// Label for capture-compare register 1
pub struct CCR1;
/// Label for capture-compare register 2
pub struct CCR2;
/// Label for capture-compare register 3
pub struct CCR3;
/// Label for capture-compare register 4
pub struct CCR4;
/// Label for capture-compare register 5
pub struct CCR5;
/// Label for capture-compare register 6
pub struct CCR6;

macro_rules! ccrn_impl {
    ($TAx:ident, $CCRn:ident, $taxcctln:ident, $taxccrn:ident) => {
        impl CCRn<$CCRn> for pac::$TAx {
            #[inline(always)]
            fn set_ccrn(&self, count: u16) {
                self.$taxccrn.write(|w| unsafe { w.bits(count) });
            }

            #[inline(always)]
            fn get_ccrn(&self) -> u16 {
                self.$taxccrn.read().bits()
            }

            #[inline(always)]
            fn config_outmod(&self, outmod: Outmod) {
                self.$taxcctln.write(|w| w.outmod().bits(outmod as u8));
            }

            #[inline(always)]
            fn config_cap_mode(&self, cm: Cm, ccis: Ccis) {
                self.$taxcctln.write(|w| {
                    w.cap()
                        .capture()
                        .scs()
                        .sync()
                        .cm()
                        .bits(cm as u8)
                        .ccis()
                        .bits(ccis as u8)
                });
            }

            #[inline(always)]
            fn ccifg_rd(&self) -> bool {
                self.$taxcctln.read().ccifg().bit()
            }

            #[inline(always)]
            fn ccifg_clr(&self) {
                self.$taxcctln.modify(|_,w| w.ccifg().clear_bit());
            }

            #[inline(always)]
            fn ccie_set(&self) {
                self.$taxcctln.modify(|_,w| w.ccie().set_bit());
            }

            #[inline(always)]
            fn ccie_clr(&self) {
                self.$taxcctln.modify(|_,w| w.ccie().clear_bit());
            }

            #[inline(always)]
            fn cov_ccifg_rd(&self) -> (bool, bool) {
                let cctl = self.$taxcctln.read();
                (cctl.cov().bit(), cctl.ccifg().bit())
            }

            #[inline(always)]
            fn cov_ccifg_clr(&self) {
                self.$taxcctln
                    .modify(|_,w| w.ccifg().clear_bit().cov().clear_bit());
            }
        }
    };
}

macro_rules! timera_impl {
    ($TAx:ident, $tax:ident, $taxctl:ident, $taxex:ident, $taxiv:ident, $([$CCRn:ident, $taxcctln:ident, $taxccrn:ident]),*) => {
        impl Steal for pac::$TAx {
            #[inline(always)]
            unsafe fn steal() -> Self {
                pac::Peripherals::conjure().$TAx
            }
        }

        impl TimerA for pac::$TAx {
            #[inline(always)]
            fn reset(&self) {
                self.$taxctl.modify(|_,w| w.taclr().set_bit());
            }

            #[inline(always)]
            fn upmode(&self) {
                self.$taxctl.modify(|r, w| {
                    unsafe { w.bits(r.bits()) }
                        .taclr()
                        .set_bit()
                        .taifg()
                        .clear_bit()
                        .mc()
                        .up()
                });
            }

            #[inline(always)]
            fn continuous(&self) {
                self.$taxctl.modify(|r, w| {
                    unsafe { w.bits(r.bits()) }
                        .taclr()
                        .set_bit()
                        .taifg()
                        .clear_bit()
                        .mc()
                        .continuous()
                });
            }

            #[inline(always)]
            fn config_clock(&self, tassel: Tassel, div: TimerDiv) {
                self.$taxctl
                    .write(|w| w.tassel().bits(tassel as u8).id().bits(div as u8));
            }

            #[inline(always)]
            fn is_stopped(&self) -> bool {
                self.$taxctl.read().mc().is_stop()
            }

            #[inline(always)]
            fn stop(&self) {
                self.$taxctl.modify(|_,w| w.mc().stop());
            }

            #[inline(always)]
            fn set_taidex(&self, taidex: TimerExDiv) {
                self.$taxex.write(|w| w.taidex().bits(taidex as u8));
            }

            #[inline(always)]
            fn taifg_rd(&self) -> bool {
                self.$taxctl.read().taifg().bit()
            }

            #[inline(always)]
            fn taifg_clr(&self) {
                self.$taxctl.modify(|_,w| w.taifg().clear_bit());
            }

            #[inline(always)]
            fn taie_set(&self) {
                self.$taxctl.modify(|_,w| w.taie().set_bit());
            }

            #[inline(always)]
            fn taie_clr(&self) {
                self.$taxctl.modify(|_,w| w.taie().clear_bit());
            }

            #[inline(always)]
            fn taxiv_rd(&self) -> u16 {
                self.$taxiv.read().bits()
            }
        }

        $(ccrn_impl!($TAx, $CCRn, $taxcctln, $taxccrn);)*
    };
}

timera_impl!(
    TA0,
    ta0,
    ta0ctl,
    ta0ex0,
    ta0iv,
    [CCR0, ta0cctl0, ta0ccr0],
    [CCR1, ta0cctl1, ta0ccr1],
    [CCR2, ta0cctl2, ta0ccr2]
);

timera_impl!(
    TA1,
    ta1,
    ta1ctl,
    ta1ex0,
    ta1iv,
    [CCR0, ta1cctl0, ta1ccr0],
    [CCR1, ta1cctl1, ta1ccr1],
    [CCR2, ta1cctl2, ta1ccr2]
);

timera_impl!(
    TA2,
    ta2,
    ta2ctl,
    ta2ex0,
    ta2iv,
    [CCR0, ta2cctl0, ta2ccr0],
    [CCR1, ta2cctl1, ta2ccr1],
    [CCR2, ta2cctl2, ta2ccr2]
);

timera_impl!(
    TA3,
    ta3,
    ta3ctl,
    ta3ex0,
    ta3iv,
    [CCR0, ta3cctl0, ta3ccr0],
    [CCR1, ta3cctl1, ta3ccr1],
    [CCR2, ta3cctl2, ta3ccr2]
    // [CCR3, ta3cctl3, ta3ccr3],
    // [CCR4, ta3cctl4, ta3ccr4],
    // [CCR5, ta3cctl5, ta3ccr5],
    // [CCR6, ta3cctl6, ta3ccr6]
);

// timera_impl!(
//     TB0,
//     tb0,
//     tb0ctl,
//     tb0ex0,
//     tb0iv,
//     [CCR0, tb0cctl0, tb0ccr0],
//     [CCR1, tb0cctl1, tb0ccr1],
//     [CCR2, tb0cctl2, tb0ccr2]
// );
