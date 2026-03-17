use stm32f4xx_hal::adc::{self, Adc};
use stm32f4xx_hal::hal_02::adc::{Channel, OneShot};
use stm32f4xx_hal::nb;
use stm32f4xx_hal::pac;
use stm32f4xx_hal::rcc::Rcc;

pub type Adc1 = Adc<pac::ADC1>;

pub fn init_adc1(adc1: pac::ADC1, rcc: &mut Rcc, config: adc::config::AdcConfig) -> Adc1 {
    Adc::new(adc1, true, config, rcc)
}

pub fn read_nb<PIN>(adc: &mut Adc1, pin: &mut PIN) -> nb::Result<u16, ()>
where
    PIN: Channel<pac::ADC1, ID = u8>,
{
    OneShot::read(adc, pin)
}

pub fn read_blocking<PIN>(adc: &mut Adc1, pin: &mut PIN) -> Result<u16, ()>
where
    PIN: Channel<pac::ADC1, ID = u8>,
{
    nb::block!(OneShot::read(adc, pin))
}
