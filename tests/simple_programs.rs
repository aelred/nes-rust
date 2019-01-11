use nes_rust::CPU;
use nes_rust::mem;
use nes_rust::OpCode::*;
use nes_rust::Address;

const HALT_ADDRESS: Address = Address::new(0xDEAD);

macro_rules! run {
    ($result: expr; $( $expr: expr ),*) => {
        let mut cpu = CPU::with_memory(mem!($($expr),*));
        let result = run(&mut cpu);
        assert_eq!(result, $result);
    };
}

fn run(cpu: &mut CPU) -> u8 {
    while cpu.read(HALT_ADDRESS) == 0 {
        cpu.run_instruction();
    }

    cpu.accumulator()
}

#[test]
fn one_plus_one() {
    run!(2;
        LDAImmediate, 1u8,
        ADCImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}
