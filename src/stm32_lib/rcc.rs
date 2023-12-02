pub mod rcc{
    
    use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
    // use panic_abort as _; // requires nightly
    // use panic_itm as _; // logs messages over ITM; requires ITM support
    // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

    use cortex_m_rt::entry;
    use stm32f4::stm32f446;

    pub fn initialize_clock(){
        let dp = stm32f446::Peripherals::take().unwrap();

        dp.RCC.cr.write(|w| w.hseon().set_bit()); // Turning on the high speed external oscillator
        while dp.RCC.cr.read().hserdy().bit(){ // waiting for the high speed external oscillator ready bit to be set
            // Do noting while the hserdy bit is not set (i.e 0)
        }
    
        // Set the POWER ENABLE CLOCK and VOLTAGE REGULATOR
        dp.RCC.apb1enr.write(|w| w.pwren().set_bit()); //setting power enable clock bit
        dp.PWR.cr.write(|w| unsafe{w.vos().bits(0b10)});
    
        //Configure the Flas and Prefetch and the latency related settings
        dp.FLASH.acr.write(|w| unsafe{w.icen().set_bit()});
        dp.FLASH.acr.write(|w| unsafe{w.dcen().set_bit()});
        dp.FLASH.acr.write(|w| unsafe{w.prften().set_bit()});
        dp.FLASH.acr.write(|w| unsafe{w.latency().bits(0b0101)});
    
        // Configure AHB, APB1 and APB2 using the RCC_CFGR (division factor prescalers)
        dp.RCC.cfgr.write(|w| unsafe{w.ppre1().bits(0b100)}); //writing 4 to APB1 
        dp.RCC.cfgr.write(|w| unsafe{w.ppre2().bits(0b010)}); //writing 2 to APB2
    
        //Configure the Main PLL
        dp.RCC.pllcfgr.write(|w| unsafe {w.pllm().bits(0b000100)});  // setting PLLM to 4
        dp.RCC.pllcfgr.write(|w| unsafe {w.plln().bits(0b010110100)}); // setting PLLN to 180
        dp.RCC.pllcfgr.write(|w| w.pllp().bits(0b01)); //setting PLLP to 4. i.e option 01
    
        //Enable the PLL and wait for it to be ready
        dp.RCC.cr.write(|w| w.pllon().set_bit());
        while dp.RCC.cr.read().pllrdy().bit(){
            //Do noting while pllrdy bit is not set
        }
    
        //Select the clock source and wait for it to be on
        dp.RCC.cfgr.write(|w| unsafe {w.sw().bits(0b10)});
        while dp.RCC.cfgr.read().sws().bits() != 0b10{
            //do noting while clock source is not set
        }
    
        //Testing by turning on USER_LED on PA5
        //Enable clock to GPIOA
        dp.RCC.ahb1enr.write(|w| w.gpioaen().set_bit());
        //Configure PA5 as output
        dp.GPIOA.moder.write(|w| unsafe {w.moder5().bits(0b01)});
        //set PA5 Output to high signalling end of configuration
        dp.GPIOA.odr.write(|w| w.odr5().set_bit());
    }
}