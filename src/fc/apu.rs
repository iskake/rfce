use log::debug;

struct PulseChannel {
    channel_num: usize,

    duty_cycle: u8,
    length_counter_halt: bool,
    const_vol_env_flag: bool,
    vol_env_div_period: u8,

    sweep_enabled: bool,
    sweep_div_period: u8,
    sweep_negate: bool,
    sweep_shift_count: u8,
    sweep_reload: bool,

    timer: u16,
    length_counter_load: u8,

    sequencer: u8,
}

impl PulseChannel {
    fn new(channel_num: usize) -> PulseChannel {
        PulseChannel {
            channel_num,
            duty_cycle: 0,
            length_counter_halt: false,
            const_vol_env_flag: false,
            vol_env_div_period: 0,
            sweep_enabled: false,
            sweep_div_period: 0,
            sweep_negate: false,
            sweep_shift_count: 0,
            sweep_reload: false,
            timer: 0,
            length_counter_load: 0,
            sequencer: 0,
        }
    }

    fn write_0(&mut self, val: u8) {
        self.duty_cycle          = (val & 0b1100_0000) >> 6;
        self.length_counter_halt =  val & 0b0010_0000 != 0;
        self.const_vol_env_flag  =  val & 0b0001_0000 != 0;
        self.vol_env_div_period  =  val & 0b0000_1111;

        // TODO: Side effect: "The duty cycle is changed, but the sequencer's current position isn't affected."
        debug!("Wrote {:02x} to APU PULSE{} ${:04x} (duty cycle, ...)", val, self.channel_num, 0x4000 + (self.channel_num - 1) * 4);
    }

    fn write_1(&mut self, val: u8) {
        self.sweep_enabled     =  val & 0b1000_0000 != 0;
        self.sweep_div_period  = (val & 0b0111_0000) >> 4;
        self.sweep_negate      =  val & 0b0000_1000 != 0;
        self.sweep_shift_count =  val & 0b1000_0111;

        // Side effect: "Sets the reload flag"
        self.sweep_reload = true;

        debug!("Wrote {:02x} to APU PULSE{} ${:04x} (sweep)", val, self.channel_num, 0x4000 + (self.channel_num - 1) * 4 + 1);
    }

    fn write_2(&mut self, val: u8) {
        self.timer = (self.timer & 0x700) | val as u16;

        debug!("Wrote {:02x} to APU PULSE{} ${:04x} (timer low)", val, self.channel_num, 0x4000 + (self.channel_num - 1) * 4 + 2);
    }

    fn write_3(&mut self, val: u8) {
        self.timer = (self.timer & 0x0ff) | (val as u16 & 0b111) << 8;
        self.length_counter_load = (val & 0b1111_1000) >> 3;

        // TODO: Side effect:
        //   "The sequencer is immediately restarted at the first value of the current sequence.
        //   The envelope is also restarted. The persiod divider is _not_ reset."

        debug!("Wrote {:02x} to APU PULSE{} ${:04x} (length counter, timer)", val, self.channel_num, 0x4000 + (self.channel_num - 1) * 4 + 3);
    }
}

struct TriangleChannel {
    timer: u16,
    linear_counter: u8,
    length_counter: u8,
    linear_counter_reload_val: u8,
    length_counter_reload_val: u8,
    linear_counter_reload: bool,
    control_length_halt: bool,
    sequencer: u8,
}

impl TriangleChannel {
    fn new() -> TriangleChannel {
        TriangleChannel {
            timer: 0,
            linear_counter: 0,
            length_counter: 0,
            linear_counter_reload_val: 0,
            length_counter_reload_val: 0,
            linear_counter_reload: false,
            control_length_halt: false,
            sequencer: 0,
        }
    }

    fn write_8(&mut self, val: u8) {
        self.control_length_halt = val & 0b1000_0000 != 0;

        let counter_reload_val = val & 0b0111_1111;

        if self.control_length_halt {
            self.linear_counter_reload_val = counter_reload_val;
        } else {
            self.length_counter_reload_val = counter_reload_val;
        }

        debug!("Wrote {:02x} to APU TRIANGLE $4008 (linear counter setup)", val);
    }

    fn write_a(&mut self, val: u8) {
        self.timer = (self.timer & 0x700) | val as u16;

        debug!("Wrote {:02x} to APU TRIANGLE $400a (timer low)", val);
    }

    fn write_b(&mut self, val: u8) {
        self.timer = (self.timer & 0x0ff) | (val as u16 & 0b111) << 8;

        // Side effect: "Sets the linear counter reload flag"
        self.linear_counter_reload = true;

        debug!("Wrote {:02x} to APU TRIANGLE $400b (length counter load, timer high)", val);
    }
}

struct NoiseChannel {
    envelope_vol_div_period: u8,
    envelope_const_vol: bool,
    length_counter_halt: bool,
    mode: bool,
    period: u16,
    period_shift: u16,
    length_counter_reload_val: u8,
}

const NOISE_PERIOD_VALUES: [u16; 16] = [ 4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068 ];

impl NoiseChannel {
    fn new() -> NoiseChannel {
        NoiseChannel {
            envelope_vol_div_period: 0,
            envelope_const_vol: false,
            length_counter_halt: false,
            mode: false,
            period: NOISE_PERIOD_VALUES[0], // ?
            period_shift: 0x0000,           // ?
            length_counter_reload_val: 0,
        }
    }

    fn write_c(&mut self, val: u8) {
        // Length counter halt, constant volume/envelope flag, and volume/envelope divider period
        self.length_counter_halt     = val & 0b0010_0000 != 0;
        self.envelope_const_vol      = val & 0b0001_0000 != 0;
        self.envelope_vol_div_period = val & 0b0000_1111;

        debug!("Wrote {:02x} to APU NOISE $400c (len counter halt, ...)", val);
    }

    fn write_e(&mut self, val: u8) {
        // Mode and period
        self.mode   = val & 0b1000_0000 != 0;
        self.period = NOISE_PERIOD_VALUES[val as usize & 0b0000_1111];

        debug!("Wrote {:02x} to APU NOISE $400e (mode, period)", val);
    }

    fn write_f(&mut self, val: u8) {
        // Length counter (re)load and envelope restart
        self.length_counter_reload_val = (val & 0b1111_1000) >> 3;

        // TODO: "envelope restart"

        debug!("Wrote {:02x} to APU NOISE $400f (length counter load)", val);
    }
}

struct DMC {
    irq_enabled: bool,
    loop_flag: bool,
    period: u16,
    output_level: u8,
    sample_address: u16,
    sample_length: u16
}

const DMC_RATE_VALUES: [u16; 16] = [ 428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54 ];

impl DMC {
    fn new() -> DMC {
        DMC {
            irq_enabled: false,
            loop_flag: false,
            period: DMC_RATE_VALUES[0], // ?
            output_level: 0,
            sample_address: 0xc000,
            sample_length: 1, // ?
        }
    }

    fn write_0(&mut self, val: u8) {
        // Flags and rate
        self.irq_enabled = val & 0b1000_0000 != 0;
        self.loop_flag   = val & 0b0100_0000 != 0;
        self.period      = DMC_RATE_VALUES[val as usize & 0b0000_1111];

        debug!("Wrote {:02x} to APU DMC $4010 (irq, flags, rate)", val);
    }

    fn write_1(&mut self, val: u8) {
        // Direct load
        self.output_level = val & 0b0111_1111;

        // TODO: "If the timer is outputting a clock at the same time, the output level is occasionally not changed properly."

        debug!("Wrote {:02x} to APU DMC $4011 (direct load)", val);
    }

    fn write_2(&mut self, val: u8) {
        self.sample_address = 0xc000 | ((val as u16) << 6);

        debug!("Wrote {:02x} to APU DMC $4012 (sample address)", val);
    }

    fn write_3(&mut self, val: u8) {
        self.sample_length = ((val as u16) << 4) | 1;

        debug!("Wrote {:02x} to APU DMC $4013 (sample length)", val);
    }
}

struct FrameCounter {
    mode_5_step: bool,
    irq_enable: bool
}

impl FrameCounter {
    fn new() -> FrameCounter {
        FrameCounter {
            mode_5_step: false, // ?
            irq_enable: true,   // ?
        }
    }

    fn write_frame_counter(&mut self, val: u8) {
        // Set mode and interrupt
        self.mode_5_step = val & 0b1000_0000 != 0;

        // TODO? "If set, the frame interrupt flag is cleared, otherwise it is unaffected"
        self.irq_enable = val & 0b0100_0000 == 0;

        // TODO: Side effects:
        //   "After 3 or 4 CPU clock cycles*, the timer is reset."
        //     "* If the write occurs during an APU cycle, the effects occur 3 CPU cycles after the $4017 write cycle,
        //        and if the write occurs between APU cycles, the effects occurs 4 CPU cycles after the write cycle. "
        //   "If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated."
    }
}

struct APUStatus {
    dmc_enabled: bool,
    noise_enabled: bool,
    triangle_enabled: bool,
    pulse1_enabled: bool,
    pulse2_enabled: bool,
}

pub struct APU {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DMC,
    status: APUStatus,
    frame_counter: FrameCounter,
    cycles: usize,
}

impl APU {
    pub fn new() -> APU {
        // TODO: initial values
        APU {
            pulse1: PulseChannel::new(1),
            pulse2: PulseChannel::new(2),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DMC::new(),
            frame_counter: FrameCounter::new(),
            status: APUStatus {
                dmc_enabled: false,
                noise_enabled: false,
                triangle_enabled: false,
                pulse2_enabled: false,
                pulse1_enabled: false,
            },
            cycles: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.cycles += 1;

        // TODO: do things

        // "Triangle channel's timer is clocked on every CPU cycle"
        // triangle.tick();

        // Every _other_ cycle, starting at 1
        if self.cycles & 1 == 0 {
            // "Pulse, noise, and DMC timers are clocked on every second CPU cycle and thus produce only even periods"
            // self.pulse1.tick();
            // self.pulse2.tick();
            // self.noise.tick();
            // self.dmc.tick();
        }

        // Frame counter
        
    }

    pub fn read_addr(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => self.read_status(),
            // 0x4000-0x4014, 0x4017
            _ => 0xff, // TODO: should be open bus
        }
    }

    pub fn write_addr(&mut self, addr: u16, val: u8) {
        match addr {
            // Pulse 1
            0x4000 => self.pulse1.write_0(val),
            0x4001 => self.pulse1.write_1(val),
            0x4002 => self.pulse1.write_2(val),
            0x4003 => self.pulse1.write_3(val),
            // Pulse 2
            0x4004 => self.pulse2.write_0(val),
            0x4005 => self.pulse2.write_1(val),
            0x4006 => self.pulse2.write_2(val),
            0x4007 => self.pulse2.write_3(val),
            // Triangle
            0x4008 => self.triangle.write_8(val),
            0x4009 => {}
            0x400a => self.triangle.write_a(val),
            0x400b => self.triangle.write_b(val),
            // Noise
            0x400c => self.noise.write_c(val),
            0x400d => {}
            0x400e => self.noise.write_e(val),
            0x400f => self.noise.write_f(val),
            // DMC
            0x4010 => self.dmc.write_0(val),
            0x4011 => self.dmc.write_1(val),
            0x4012 => self.dmc.write_2(val),
            0x4013 => self.dmc.write_3(val),
            // Other
            0x4015 => self.write_status(val),
            0x4017 => self.frame_counter.write_frame_counter(val),
            _ => unreachable!(),
        }
    }

    fn read_status(&self) -> u8 {
        // let bit0 = ...;
        // let bit1 = ...;
        // let bit2 = ...;
        // let bit3 = ...;
        // let bit4 = ...;
        // let bit5 = 1; // TODO: open bus read ("the open bus value comes from the last cycle that did not read $4015")
        // let bit6 = ...;
        // let bit7 = ...;
        // TODO
        0xff
    }

    fn write_status(&mut self, val: u8) {
        self.status.pulse1_enabled   = (val & 0b00001) != 0;
        self.status.pulse2_enabled   = (val & 0b00010) != 0;
        self.status.triangle_enabled = (val & 0b00100) != 0;
        self.status.noise_enabled    = (val & 0b01000) != 0;
        self.status.dmc_enabled      = (val & 0b10000) != 0;

        debug!("Wrote {:02x} to APU STATUS ($4015)", val & 0b11111)
    }
}
