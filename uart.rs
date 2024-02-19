use embassy :: executor :: Spawner ;
use embassy_stm32 :: usart :: {Config, Uart};
use embassy_stm32 :: dma :: {NoDma, DmaConfig};
use embassy_stm32 :: gpio :: {Input, Output};
use stm32g4::stm32g431::{};

const rx_buff_size: usize = 25;

static mut rx_buff: [u8; rx_buff_size] = [0; rx_buff_size];
static rx_buff_index: forever<core::cell::Cell<usize>> = forever::new(core::cell::Cell::new(0));

async fn main(_spawner: spawner){
    let p = peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc);
    let uart_pins = signal::new (
        gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh), // PA9 as RX
        gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh), // PA10 as TX
    ).into();

  let mut uart = uart::new(USART2, uart_pins, interrupt:: take!(USART2));

  let mut config = Config::default();
    config.baudrate = 9600; 
    config.parity = usart::Parity::None;
    config.invert_rx = 1; //invert the RX signal
    uart.init(&mut config, interrupt::take!(USART2), &mut rcc);

    rx_buff_index.put(core::cell::Cell::new(0));
   
   loop{
        let byte = uart.receive(&mut rx_buff).await.unwrap();

        //print this shit
        for i in 0 ..byte{
           uart.write()


        }
    }





}

