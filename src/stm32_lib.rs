pub mod rcc {
    use panic_halt as _;
    use stm32f4::stm32f446; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                            // use panic_abort as _; // requires nightly
                            // use panic_itm as _; // logs messages over ITM; requires ITM support
                            // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

    pub fn initialize_clock(
        rcc: &mut stm32f446::RCC,
        pwr: &mut stm32f446::PWR,
        flash: &mut stm32f446::FLASH,
    ) {
        rcc.cr.write(|w| w.hseon().set_bit()); // Turning on the high speed external oscillator
        while rcc.cr.read().hserdy().bit() { // waiting for the high speed external oscillator ready bit to be set
             // Do noting while the hserdy bit is not set (i.e 0)
        }

        // Set the POWER ENABLE CLOCK and VOLTAGE REGULATOR
        rcc.apb1enr.write(|w| w.pwren().set_bit()); //setting power enable clock bit
        pwr.cr.write(|w| unsafe { w.vos().bits(0b10) });

        //Configure the Flas and Prefetch and the latency related settings
        flash.acr.write(|w| w.icen().set_bit());
        flash.acr.modify(|_r, w| w.dcen().set_bit());
        flash.acr.modify(|_r, w| w.prften().set_bit());
        flash.acr.modify(|_r, w| w.latency().bits(0b0101));

        // Configure AHB, APB1 and APB2 using the RCC_CFGR (division factor prescalers)
        rcc.cfgr.write(|w| unsafe { w.ppre1().bits(0b100) }); //writing 4 to APB1
        rcc.cfgr.modify(|_r, w| unsafe { w.ppre2().bits(0b010) }); //writing 2 to APB2

        //Configure the Main PLL
        rcc.pllcfgr.write(|w| unsafe { w.pllm().bits(0b000100) }); // setting PLLM to 4
        rcc.pllcfgr
            .modify(|_r, w| unsafe { w.plln().bits(0b010110100) }); // setting PLLN to 180
        rcc.pllcfgr.modify(|_r, w| w.pllp().bits(0b01)); //setting PLLP to 4. i.e option 01

        //Enable the PLL and wait for it to be ready
        rcc.cr.modify(|_r, w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit() {
            //Do noting while pllrdy bit is not set
        }

        //Select the clock source and wait for it to be on
        rcc.cfgr.modify(|_r, w| unsafe { w.sw().bits(0b10) });
        while rcc.cfgr.read().sws().bits() != 0b10 {
            //do noting while clock source is not set
        }
    }

    // delay function implemented using timer 11
    pub fn delay_us(us: u16, timer: &mut stm32f446::TIM11) {
        timer.cnt.write(|w| unsafe { w.cnt().bits(0x0000) });
        while timer.cnt.read().cnt().bits() < us {}
    }

    pub fn delay_ms(ms: u16, timer: &mut stm32f446::TIM11) {
        let mut i = 0;
        for i in 0..ms {
            delay_us(1000, timer);
        }
    }
}

pub mod lcd {

    use crate::comm::i2c;
    use stm32f4::stm32f446;

    pub fn lcd_write(i2c: &mut stm32f446::I2C1, data: &[u8; 4]) {
        for i in 0..4 {
            i2c::i2c_write(data[i], i2c);
        }
    }

    pub fn lcd_write_str(i2c: &mut stm32f446::I2C1, input: &str) {
        for byte in input.bytes() {
            lcd_send_data(byte, i2c);
        }
    }

    pub fn lcd_send_cmd(cmd: u8, i2c: &mut stm32f446::I2C1) {
        let data_u: u8;
        let data_l: u8;
        let temp: u8;
        data_u = cmd & 0xf0;
        temp = cmd << 4;
        data_l = temp & 0xf0;
        let data: [u8; 4] = [
            data_u | 0x0C, // en=1, rs=0
            data_u | 0x08, // en=0, rs=0
            data_l | 0x0C, // en=1, rs=0
            data_l | 0x08, // en=0, rs=0
        ];
        lcd_write(i2c, &data);
    }

    pub fn lcd_send_data(data: u8, i2c: &mut stm32f446::I2C1) {
        let data_u: u8;
        let data_l: u8;
        let temp: u8;
        data_u = data & 0xf0;
        temp = data << 4;
        data_l = temp & 0xf0;
        let data_t: [u8; 4] = [
            data_u | 0x0D, // en=1, rs=0
            data_u | 0x09, // en=0, rs=0
            data_l | 0x0D, // en=1, rs=0
            data_l | 0x09, // en=0, rs=0
        ];
        lcd_write(i2c, &data_t);
    }

    pub fn lcd_clear(i2c: &mut stm32f446::I2C1) {
        lcd_send_cmd(0x80, i2c);
        for _ in 0..70 {
            lcd_send_data(' ' as u8, i2c);
        }
    }

    pub fn lcd_put_cur(i2c: &mut stm32f446::I2C1, row: u8, mut col: u8) {
        match row {
            0 => col |= 0x80,
            1 => col |= 0xc0,
            _ => {}
        }
        lcd_send_cmd(col, i2c)
    }

    pub fn lcd_init(i2c: &mut stm32f446::I2C1, timer: &mut stm32f446::TIM11) {
        // 4 bit initialisation
        rcc::delay_ms(50, timer); //wait for >40ms
        lcd_send_cmd(0x30, i2c);
        rcc::delay_ms(5, timer); //wait for >4.1ms
        lcd_send_cmd(0x30, i2c);
        rcc::delay_us(150, timer); //wait for >100ms
        lcd_send_cmd(0x30, i2c);
        rcc::delay_ms(10, timer); //wait for >4.1ms
        lcd_send_cmd(0x20, i2c); //4bit mode
        rcc::delay_ms(10, timer);

        //display initialisation
        lcd_send_cmd(0x28, i2c); // function set ---> DL=0 (4 bit mode), N = 1 (2 line display) F = 0 (5*8 characters)
        rcc::delay_ms(1, timer);
        lcd_send_cmd(0x08, i2c); // Display on/off control --> D=0, C=0, B=0 ---> display off
        rcc::delay_ms(1, timer);
        lcd_send_cmd(0x01, i2c); // clear display
        rcc::delay_ms(2, timer);
        lcd_send_cmd(0x06, i2c); // Entry mode set --> I/D = 1 (increment cursor) & S = 0 (no shift)
        rcc::delay_ms(1, timer);
        lcd_send_cmd(0x0C, i2c); // Display on/off control --> D=1, C and B = 0. (Cursor and blink last two bits)
        rcc::delay_ms(1, timer);
    }
}

pub mod comm {
    pub mod i2c {
        use stm32f4::stm32f446;

        pub fn i2c_config(
            rcc: &mut stm32f4::stm32f446::RCC,
            gpio: &mut stm32f4::stm32f446::GPIOB,
            i2c: &mut stm32f4::stm32f446::I2C1,
        ) {
            //Enable 12c and gpio clocks
            rcc.apb1enr.write(|w| w.i2c1en().set_bit());
            rcc.ahb1enr.write(|w| w.gpioben().set_bit());

            //configure gpio pins

            gpio.moder.write(|w| w.moder8().bits(0b10)); // place pin PB8 and PB9 in alternate fucntion mode
            gpio.moder.modify(|_r, w| w.moder9().bits(0b10));

            gpio.otyper.modify(|_r, w| w.ot8().set_bit()); // enabling open drain mode for pin PB8 and PB9
            gpio.otyper.modify(|_r, w| w.ot9().set_bit());

            gpio.ospeedr.modify(|_r, w| w.ospeedr8().bits(0b11)); // setting the pins PB8 and PB9 to high speed (fastest)
            gpio.ospeedr.modify(|_r, w| w.ospeedr9().bits(0b11));

            gpio.pupdr.modify(|_r, w| unsafe { w.pupdr8().bits(0b01) }); //setting the pull up resistors for pinis PB8 and PB9
            gpio.pupdr.modify(|_r, w| unsafe { w.pupdr9().bits(0b01) });

            gpio.afrh.modify(|_r, w| w.afrh8().bits(0b0100)); //setting the alternate mode to i2c for pins PB8 and PB9
            gpio.afrh.modify(|_r, w| w.afrh9().bits(0b0100));

            i2c.cr1.write(|w| w.swrst().set_bit()); // put i2c in the reset state
            i2c.cr1.modify(|_r, w| w.swrst().clear_bit()); //take i2c out of reset state

            // Program the peripheral input clock in I2c_cr2 register in order to generate correct timings
            i2c.cr2.write(|w| unsafe { w.freq().bits(0b101101) }); //setting periphercal input clock fequency to 45mhz (current max value of APB)
            i2c.cr2.modify(|_r, w| w.itevten().enabled()); //enable even interrupts

            //configure the clock control registers
            i2c.ccr.write(|w| w.f_s().standard());
            i2c.ccr.modify(|_r, w| w.duty().duty2_1());
            i2c.ccr.modify(|_r, w| unsafe { w.ccr().bits(0b11100001) }); //setting crr to 225 (calculated see manual)
            i2c.trise.modify(|_r, w| w.trise().bits(0b101110)); // configure the rise time register (calculated see manual)
            i2c.cr1.modify(|_r, w| w.pe().set_bit()); // Enable the peripheral in i2c_cr1 register
        }

        pub fn timer_config(
            rcc: &mut stm32f4::stm32f446::RCC,
            timer: &mut stm32f4::stm32f446::TIM11,
        ) {
            //Enable Timer clock
            rcc.apb2enr.modify(|_r, w| w.tim11en().set_bit());
            //initialize timer for delay
            // Set the prescaler and the ARR
            timer.psc.modify(|_r, w| w.psc().bits(0b0000000010110011)); //180MHz/180 = 1MHz ~ 1us, prescalar set to 179, ie. 179+1 = 180;
            timer.arr.modify(|_r, w| unsafe { w.arr().bits(0xffff) });

            //Enable the Timer, and wait for the update Flag to set
            timer.cr1.modify(|_r, w| w.cen().set_bit());
            timer.sr.read().uif().bit();
            while !timer.sr.read().uif().bit() {}
        }

        pub fn i2c_start(i2c: &mut stm32f446::I2C1) {
            //send the Start condition
            while i2c.sr2.read().busy().bit_is_set() {}
            i2c.cr1.modify(|_r, w| w.start().set_bit()); // setting the start bit

            while !i2c.sr1.read().sb().bit_is_set() { // waiting for the start condition
            }
        }

        pub fn i2c_stop(i2c: &mut stm32f446::I2C1) {
            i2c.cr1.write(|w| w.stop().set_bit());
        }

        pub fn i2c_address(address: u8, i2c: &mut stm32f446::I2C1) {
            //send the slave address to the DR register
            i2c.dr.write(|w| w.dr().bits(address));
            while !i2c.sr1.read().addr().is_match() {}
            let _temp = i2c.sr1.read(); //reading sr1 and sr2 to clear the addr bit
            let _temp2 = i2c.sr2.read();
        }

        pub fn i2c_write(data: u8, i2c: &mut stm32f446::I2C1) {
            //wait for the TXE bit 7 in cr1 to set. This indicates that the DR is empty
            while !i2c.sr1.read().tx_e().is_empty() {}
            //load the data in the data register
            i2c.dr.write(|w| w.dr().bits(data));
            //wait for data transfer to finish
            while !i2c.sr1.read().btf().is_finished() {}
        }
    }
}
