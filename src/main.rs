mod compute;
use compute::run;

fn main() {
    pollster::block_on(run())
}
