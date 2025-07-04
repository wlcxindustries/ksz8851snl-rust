use crate::registers::*;
use bondrewd::Bitfields;
use bytemuck::Zeroable;
use embedded_hal::spi::{self, ErrorKind};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::{Operation, SpiDevice};
use embedded_registers::spi::{CodecAsync, SpiDeviceAsync};
use embedded_registers::{Register, RegisterInterfaceAsync};

#[repr(u8)]
enum Opcode {
    RegRead = 0b00,
    RegWrite = 0b01,
    RXRead = 0b10,
    TXWrite = 0b11,
}

const CHIP_ID_FAMILY: u8 = 0x88;
const CHIP_ID_CHIP: u8 = 0x7;

/// Implements [`CodecAsync`] to allow use of [`SpiDeviceAsync`]
pub struct KSZ8851Codec {}

impl CodecAsync for KSZ8851Codec {
    async fn read_register<R, I>(interface: &mut I) -> Result<R, I::Error>
    where
        R: embedded_registers::ReadableRegister,
        I: embedded_hal_async::spi::r#SpiDevice,
    {
        let mut reg = R::zeroed();
        interface
            .transaction(&mut [
                Operation::Write(&reg_cmd(
                    Opcode::RegRead,
                    R::ADDRESS.try_into().unwrap(),
                    R::REGISTER_SIZE.try_into().unwrap(),
                )),
                Operation::Read(reg.data_mut()),
            ])
            .await?;
        Ok(reg)
    }

    async fn write_register<R, I>(
        interface: &mut I,
        register: impl AsRef<R>,
    ) -> Result<(), I::Error>
    where
        R: embedded_registers::WritableRegister,
        I: embedded_hal_async::spi::r#SpiDevice,
    {
        interface
            .transaction(&mut [
                Operation::Write(&reg_cmd(
                    Opcode::RegWrite,
                    R::ADDRESS.try_into().unwrap(),
                    R::REGISTER_SIZE.try_into().unwrap(),
                )),
                Operation::Write(register.as_ref().data()),
            ])
            .await
    }
}
fn reg_cmd(o: Opcode, addr: u8, count: u8) -> [u8; 2] {
    // The device only supports accessing 4-aligned addresses, with selectable bytes
    // being read/written ("byte enables").
    let byte_enable = match (addr & 0b11, count) {
        (0, 2) => 0b0011,
        (2, 2) => 0b1100,
        (_, _) => unimplemented!(),
    };
    [
        ((o as u8) << 6) | (byte_enable << 2) | (addr >> 6),
        (addr & 0b00111100) << 2,
    ]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    SpiError(ErrorKind),
    BadChipId {
        expected_family: u8,
        actual_family: u8,
        expected_chip: u8,
        actual_chip: u8,
    },
    FailedBuiltInSelfTest {
        rx_bist_failed: bool,
        tx_bist_failed: bool,
    },
    TxPacketTooBig {
        size: usize,
        max: u16,
    },
    RxFrameInvalid,
    RxNoFrameAvailable,
}

impl<SE: spi::Error> From<SE> for Error {
    fn from(value: SE) -> Self {
        Self::SpiError(value.kind())
    }
}
pub struct Chip<SPI: SpiDevice, D: DelayNs> {
    delay: D,
    pub dev: SpiDeviceAsync<SPI, KSZ8851Codec>,
    next_frame_id: u8,
}

#[derive(Bitfields, Default)]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct TXCtrlWord {
    transmit_interrupt_on_completion: bool,
    #[bondrewd(bit_length = 9, endianness = "be")]
    __: u16,
    #[bondrewd(bit_length = 6)]
    frame_id: u8,
}

impl<SPI: SpiDevice, D: DelayNs> Chip<SPI, D> {
    /// Create a new driver from the given SPI device `dev`.
    pub fn new(dev: SPI, delay: D) -> Self {
        Self {
            delay,
            dev: SpiDeviceAsync::new(dev),
            next_frame_id: 0,
        }
    }

    /// Initialize the chip.
    ///
    /// This:
    /// - Resets the chip
    /// - Checks that it is what we think it is
    /// - Checks selftest registers

    pub async fn init(&mut self) -> Result<(), Error> {
        self.dev
            .write_register(GRR::zeroed().with_global_soft_reset(true))
            .await?;
        self.delay.delay_ms(10).await;
        self.dev.write_register(GRR::zeroed()).await?;
        self.delay.delay_ms(10).await;
        let cider = self.dev.read_register::<CIDER>().await?;
        if cider.read_chip_id() != CHIP_ID_CHIP || cider.read_family_id() != CHIP_ID_FAMILY {
            return Err(Error::BadChipId {
                expected_family: CHIP_ID_FAMILY,
                actual_family: cider.read_family_id(),
                expected_chip: CHIP_ID_CHIP,
                actual_chip: cider.read_chip_id(),
            });
        }
        #[cfg(feature = "defmt")]
        defmt::info!("Found ksz8851snl rev {}", cider.read_revision_id());
        let mbir = self.dev.read_register::<MBIR>().await?;
        if mbir.read_rx_memory_bist_fail() || mbir.read_tx_memory_bist_fail() {
            return Err(Error::FailedBuiltInSelfTest {
                rx_bist_failed: mbir.read_rx_memory_bist_fail(),
                tx_bist_failed: mbir.read_tx_memory_bist_fail(),
            });
        }

        let txfdpr = self
            .dev
            .read_register::<TXFDPR>()
            .await
            .unwrap()
            .with_tx_frame_data_pointer_auto_increment(true);
        self.dev.write_register(txfdpr).await.unwrap();

        let txcr = self
            .dev
            .read_register::<TXCR>()
            .await
            .unwrap()
            .with_checksum_gen_icmp(false)
            .with_checksum_gen_tcp(false)
            .with_checksum_gen_ip(false)
            .with_flow_control_enable(false)
            .with_padding_enable(true)
            .with_crc_enable(true);
        self.dev.write_register(txcr).await.unwrap();

        // Configure rx interrupt to be every 10ms at most. TODO: is this sufficient?
        // self.dev
        //     .write_register(RXDTTR::zeroed().with_receive_duration_timer_threshold(1000))
        //     .await
        //     .unwrap();

        let rxfdpr = self
            .dev
            .read_register::<RXFDPR>()
            .await
            .unwrap()
            .with_rx_frame_pointer_auto_increment(true);
        self.dev.write_register(rxfdpr).await.unwrap();

        let rxfctr = self
            .dev
            .read_register::<RXFCTR>()
            .await
            .unwrap()
            .with_receive_frame_count_threshold(1);
        self.dev.write_register(rxfctr).await.unwrap();
        let rxqcr = self
            .dev
            .read_register::<RXQCR>()
            .await
            .unwrap()
            .with_rx_frame_count_threshold_enable(true)
            //.with_rx_duration_timer_threshold_enable(true)
            .with_rx_ip_header_two_byte_offset_enable(true)
            .with_auto_dequeue_rxq_frame_enable(true);
        self.dev.write_register(rxqcr).await.unwrap();

        let rxcr = self
            .dev
            .read_register::<RXCR1>()
            .await
            .unwrap()
            .with_receive_udp_frame_checksum_check_enable(false)
            .with_receive_tcp_frame_checksum_check_enable(false)
            .with_receive_ip_frame_checksum_check_enable(false)
            .with_receive_flow_control_enable(true)
            // You need broadcast for ARP!
            .with_receive_broadcast_enable(true)
            .with_receive_unicast_enable(true);
        self.dev.write_register(rxcr).await.unwrap();

        let rxcr2 = self
            .dev
            .read_register::<RXCR2>()
            .await
            .unwrap()
            .with_ip4_ip6_udp_fragment_frame_pass(true)
            .with_receive_ip4_ip6_udp_frame_checksum_equal_zero(true)
            .with_udp_lite_frame_enable(true)
            .with_receive_icmp_frame_checksum_check_enable(true)
            .with_spi_receive_data_burst_length(SPIRxDataBurstLength::SINGLEFRAME);
        self.dev.write_register(rxcr2).await.unwrap();

        let ier = self
            .dev
            .read_register::<IER>()
            .await
            .unwrap()
            .with_link_change_enable(true)
            .with_transmit_space_available_enable(true)
            .with_transmit_enable(true)
            .with_receive_enable(true)
            .with_receive_overrun_enable(true)
            .with_spi_bus_error_enable(true);
        self.dev.write_register(ier).await.unwrap();

        // There are two ways to transmit - auto enqueue and manual enqueue.
        // Auto enqueue involves setting TXQCR[2] at init time, and means you can (supposedly)
        // write multiple frames at once. According to errata this doesn't work reliably?
        // Manual enqueue involves setting TXQCR[0] *after* you've written the frame to transmit.
        let txqcr = self
            .dev
            .read_register::<TXQCR>()
            .await
            .unwrap()
            .with_auto_enqueue_txq_frame_enable(false);
        self.dev.write_register(txqcr).await.unwrap();

        let txcr = self
            .dev
            .read_register::<TXCR>()
            .await
            .unwrap()
            .with_transmit_enable(true);
        self.dev.write_register(txcr).await.unwrap();

        let rxcr = self
            .dev
            .read_register::<RXCR1>()
            .await
            .unwrap()
            .with_receive_enable(true);
        self.dev.write_register(rxcr).await.unwrap();

        Ok(())
    }

    pub async fn set_leds(&mut self, on: bool) -> Result<(), Error> {
        let p1cr = self.dev.read_register::<P1CR>().await?.with_led_off(!on);
        self.dev.write_register(p1cr).await.map_err(Into::into)
    }

    /// Set the MAC address used by the chip
    pub async fn set_mac(&mut self, mac_addr: [u8; 6]) -> Result<(), Error> {
        self.dev
            .write_register(MARH::zeroed().with_marh(mac_addr[0..=1].try_into().unwrap()))
            .await?;
        self.dev
            .write_register(MARM::zeroed().with_marm(mac_addr[2..=3].try_into().unwrap()))
            .await?;
        self.dev
            .write_register(MARL::zeroed().with_marl(mac_addr[4..=5].try_into().unwrap()))
            .await?;
        Ok(())
    }

    /// Retrieve the MAC address from the chip.
    ///
    /// N.B: it doesn't come with one, so at startup this will be zeroed or garbage
    pub async fn get_mac(&mut self) -> Result<[u8; 6], Error> {
        let high = self.dev.read_register::<MARH>().await?.read_marh();
        let med = self.dev.read_register::<MARM>().await?.read_marm();
        let low = self.dev.read_register::<MARL>().await?.read_marl();
        Ok([high[0], high[1], med[0], med[1], low[0], low[1]])
    }

    /// Is the link status good (i.e. up)
    pub async fn link_good(&mut self) -> Result<bool, Error> {
        Ok(self.dev.read_register::<P1SR>().await?.read_link_good())
    }

    /// Check if the chip has space in the tx buffer to tx a packet of len `tx_len`.
    /// returns true if there's enough space, false if not. If not, also enables the
    /// chip's memory available interrupt so we're informed when there is space.
    pub async fn ready_tx(&mut self, tx_len: usize) -> Result<bool, Error> {
        if tx_len > 2000 {
            return Err(Error::TxPacketTooBig {
                size: tx_len,
                max: 2000,
            });
        }
        let available = self
            .dev
            .read_register::<TXMIR>()
            .await
            .unwrap()
            .read_txma_memory_available();
        if (tx_len + 4) > available.into() {
            // No room in the device's buffer currently
            self.dev
                .write_register(
                    TXNTFSR::zeroed().with_tx_next_total_frame_size((tx_len + 4) as u16),
                )
                .await?;
            self.dev
                .write_register(TXQCR::zeroed().with_txq_memory_available_monitor(true))
                .await?;
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// TX the given frame immediately. This assumes that we know there's enough space in
    /// the chip's tx buffer by calling having called `ready_tx` already.
    pub async fn tx(&mut self, buf: &[u8]) -> Result<(), Error> {
        // Disable interrupts
        let ier = self.dev.read_register::<IER>().await.unwrap();
        self.dev.write_register(IER::zeroed()).await.unwrap();
        // Enable TXQ write access
        let mut rxqcr = self.dev.read_register::<RXQCR>().await.unwrap();
        rxqcr.write_start_dma_access(true);
        self.dev.write_register(rxqcr).await.unwrap();

        let byte_count: [u8; 2] = (buf.len() as u16).to_le_bytes();

        let mut txc = TXCtrlWord::default();
        txc.transmit_interrupt_on_completion = true;
        txc.frame_id = self.next_frame_id;

        let _pad = (4 - (buf.len() % 4)) % 4;
        let pad = &mut [0u8; 3][0.._pad];

        self.dev
            .interface
            .transaction(&mut [
                Operation::Write(&[(Opcode::TXWrite as u8) << 6]),
                Operation::Write(&txc.into_bytes()),
                Operation::Write(&byte_count),
                Operation::Write(buf),
                Operation::Write(pad),
            ])
            .await?;
        if self.next_frame_id == 0x1f {
            self.next_frame_id = 0;
        } else {
            self.next_frame_id += 1;
        }

        // Disable TXQ write access
        let mut rxqcr = self.dev.read_register::<RXQCR>().await.unwrap();
        rxqcr.write_start_dma_access(false);
        self.dev.write_register(rxqcr).await.unwrap();

        // Manually enqueue the frame
        let mut txqcr = self.dev.read_register::<TXQCR>().await.unwrap();
        txqcr.write_manual_enqueue_txq_frame_enable(true);
        self.dev.write_register(txqcr).await.unwrap();

        // Reenable interrupts
        self.dev.write_register(ier).await.unwrap();

        Ok(())
    }

    // Get the number of RX frames ready to be read from the chip.
    // N.B. only updated on interrupt - if no interrupts are enabled this doesn't change!
    pub async fn rx_frames_available(&mut self) -> Result<u8, Error> {
        Ok(self
            .dev
            .read_register::<RXFCTR>()
            .await?
            .read_rx_frame_count())
    }

    /// Receive a single frame from the chip.
    pub async fn rx(&mut self, rx_buf: &mut [u8]) -> Result<usize, Error> {
        // Disable interrupts
        let ier = self.dev.read_register::<IER>().await?;
        assert!(!ier.read_receive_enable());
        self.dev.write_register(IER::zeroed()).await.unwrap();

        let frame_status = self.dev.read_register::<RXFHSR>().await?.read_all();
        let byte_count = self
            .dev
            .read_register::<RXFHBCR>()
            .await
            .unwrap()
            .read_receive_byte_count();
        #[cfg(feature = "defmt")]
        defmt::debug!("frame RX, {} bytes, {}", byte_count, frame_status);
        if !frame_status.frame_valid {
            // Either there is no frame or it's not done receiving.
            return Err(Error::RxNoFrameAvailable);
        }
        if frame_status.crc_error
            || frame_status.runt_frame
            || frame_status.frame_too_long
            || frame_status.mii_error
            || frame_status.udp_checksum_status
            || frame_status.tcp_checksum_status
            || frame_status.ip_checksum_status
            || frame_status.icmp_checksum_status
        {
            // Frame error - discard
            let rxqcr = self
                .dev
                .read_register::<RXQCR>()
                .await
                .unwrap()
                .with_release_rx_error_frame(true);
            self.dev.write_register(rxqcr).await.unwrap();
            // We need to wait until this is cleared before trying to rx again
            while self
                .dev
                .read_register::<RXQCR>()
                .await
                .unwrap()
                .read_release_rx_error_frame()
            {}
            return Err(Error::RxFrameInvalid);
        }
        if usize::from(byte_count) > rx_buf.len() {
            panic!("RX byte count too big!!!");
        }

        // Reset the rx frame pointer
        let rxfdpr = self.dev.read_register::<RXFDPR>().await.unwrap();
        self.dev
            .write_register(rxfdpr.with_rx_frame_pointer(0))
            .await
            .unwrap();

        // Enable DMA
        let rxqcr = self
            .dev
            .read_register::<RXQCR>()
            .await
            .unwrap()
            .with_start_dma_access(true);
        self.dev.write_register(rxqcr).await.unwrap();

        // We need to read a multiple of 4 bytes in total - so we may need some padding
        let pad = (4 - (byte_count % 4)) % 4;
        let discard = &mut [0u8; 3];

        let mut status = RXFHSR::zeroed();
        let mut bc = RXFHBCR::zeroed();

        let crc = &mut [0u8; 4];

        self.dev
            .interface
            .transaction(&mut [
                Operation::Write(&[(Opcode::RXRead as u8) << 6]),
                // 4 dummy bytes
                Operation::Read(&mut [0u8; 4]),
                // Two status word bytes
                Operation::Read(status.data_mut()),
                // Two byte count bytes
                Operation::Read(bc.data_mut()),
                // Two IP header offset bytes
                Operation::Read(&mut [0u8; 2]),
                Operation::Read(&mut rx_buf[0..(byte_count - 4 - 2) as usize]),
                Operation::Read(crc),
                Operation::Read(&mut discard[0..pad as usize]),
            ])
            .await
            .unwrap();

        #[cfg(feature = "defmt")]
        defmt::debug!("Got frame with CRC {:x}", u32::from_be_bytes(*crc));

        assert_eq!(frame_status, status.read_all());
        assert_eq!(byte_count, bc.read_receive_byte_count());

        // Disable DMA
        let rxqcr = self
            .dev
            .read_register::<RXQCR>()
            .await
            .unwrap()
            .with_start_dma_access(false);
        self.dev.write_register(rxqcr).await.unwrap();

        // Reenable interrupts
        self.dev.write_register(ier).await.unwrap();

        Ok((byte_count - 4).into())
    }
}
