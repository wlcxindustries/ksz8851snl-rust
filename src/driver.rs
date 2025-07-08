use crate::device::field_sets::{Rxfhbcr, Rxfhsr, TxCtrlWord};
use crate::device::{Ksz8851snl, Ksz8851snlInterface, SpiRxDataBurstLength};
use device_driver::FieldSet;
use embedded_hal::spi::{self, ErrorKind};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::{Operation, SpiDevice};

#[repr(u8)]
pub(crate) enum Opcode {
    RegRead = 0b00,
    RegWrite = 0b01,
    RXRead = 0b10,
    TXWrite = 0b11,
}

const CHIP_ID_FAMILY: u8 = 0x88;
const CHIP_ID_CHIP: u8 = 0x7;

pub(crate) fn reg_cmd(o: Opcode, addr: u8, count: u8) -> [u8; 2] {
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
    pub dev: Ksz8851snl<Ksz8851snlInterface<SPI>>,
    next_frame_id: u8,
}

impl<SPI: SpiDevice, D: DelayNs> Chip<SPI, D> {
    /// Create a new driver from the given SPI device `dev`.
    pub fn new(dev: SPI, delay: D) -> Self {
        Self {
            delay,
            dev: Ksz8851snl::new(Ksz8851snlInterface { bus: dev }),
            next_frame_id: 0,
        }
    }

    /// Initialize the chip.
    ///
    /// This:
    /// - Resets the chip
    /// - Checks that it is what we think it is
    /// - Checks selftest registers
    /// - Configures RX and TX functions
    /// - Enables RX and TX
    pub async fn init(&mut self) -> Result<(), Error> {
        self.dev
            .grr()
            .write_async(|grr| grr.set_global_soft_reset(true))
            .await?;
        self.delay.delay_ms(10).await;
        self.dev.grr().write_with_zero_async(|_| {}).await?;
        self.delay.delay_ms(10).await;
        let cider = self.dev.cider().read_async().await?;
        if cider.chip_id() != CHIP_ID_CHIP || cider.family_id() != CHIP_ID_FAMILY {
            return Err(Error::BadChipId {
                expected_family: CHIP_ID_FAMILY,
                actual_family: cider.family_id(),
                expected_chip: CHIP_ID_CHIP,
                actual_chip: cider.chip_id(),
            });
        }
        #[cfg(feature = "defmt")]
        defmt::info!("Found ksz8851snl rev {}", cider.revision_id());
        let mbir = self.dev.mbir().read_async().await?;
        if mbir.rxmbfa() || mbir.txmbfa() {
            return Err(Error::FailedBuiltInSelfTest {
                rx_bist_failed: mbir.rxmbfa(),
                tx_bist_failed: mbir.txmbfa(),
            });
        }

        self.dev
            .txfdpr()
            .modify_async(|r| r.set_txfpai(true))
            .await?;

        self.dev
            .txcr()
            .modify_async(|r| {
                r.set_tcgicmp(false);
                r.set_tcgtcp(false);
                r.set_tcgip(false);
                r.set_txfce(false);
                r.set_txpe(true);
                r.set_txce(true);
            })
            .await?;

        // Configure rx interrupt to be every 10ms at most. TODO: is this sufficient?
        // self.dev
        //     .write_register(RXDTTR::zeroed().with_receive_duration_timer_threshold(1000))
        //     .await
        //     .unwrap();

        self.dev
            .rxfdpr()
            .modify_async(|r| r.set_rxfpai(true))
            .await?;
        self.dev.rxfctr().modify_async(|r| r.set_rxfct(1)).await?;
        self.dev
            .rxqcr()
            .modify_async(|r| {
                r.set_rxfcte(true);
                // r.set_rxdtte(true);
                r.set_rxiphtoe(true);
                r.set_adrfe(true);
            })
            .await?;

        self.dev
            .rxcr_1()
            .modify_async(|r| {
                // Disable checksum verification
                r.set_rxudpfcc(false);
                r.set_rxtcpfcc(false);
                r.set_rxipfcc(false);

                r.set_rxfce(true);
                // You need broadcast for ARP!
                r.set_rxbe(true);
                r.set_rxue(true);
            })
            .await?;

        self.dev
            .rxcr_2()
            .modify_async(|r| {
                r.set_iufpp(true);
                r.set_rxiufcez(true);
                r.set_udplfe(true);
                r.set_rxicmpfcc(true);
                r.set_srdbl(SpiRxDataBurstLength::SingleFrame);
            })
            .await?;

        self.dev
            .ier()
            .modify_async(|r| {
                r.set_lcie(true);
                r.set_txsaie(true);
                r.set_txie(true);
                r.set_rxie(true);
                r.set_rxoie(true);
                r.set_spibeie(true);
            })
            .await?;

        // There are two ways to transmit - auto enqueue and manual enqueue.
        // Auto enqueue involves setting TXQCR[2] at init time, and means you can (supposedly)
        // write multiple frames at once. According to errata this doesn't work reliably?
        // Manual enqueue involves setting TXQCR[0] *after* you've written the frame to transmit.
        self.dev
            .txqcr()
            .modify_async(|r| r.set_aetfe(false))
            .await?;

        self.dev.txcr().modify_async(|r| r.set_txe(true)).await?;

        self.dev.rxcr_1().modify_async(|r| r.set_rxe(true)).await?;

        Ok(())
    }

    pub async fn set_leds(&mut self, on: bool) -> Result<(), Error> {
        self.dev
            .p_1_mbcr()
            .modify_async(|r| r.set_disable_led(!on))
            .await
            .map_err(Into::into)
    }

    /// Set the MAC address used by the chip
    pub async fn set_mac(&mut self, mac_addr: [u8; 6]) -> Result<(), Error> {
        self.dev
            .marh()
            .write_async(|r| {
                r.set_ma_5(mac_addr[0]);
                r.set_ma_4(mac_addr[1]);
            })
            .await?;
        self.dev
            .marm()
            .write_async(|r| {
                r.set_ma_3(mac_addr[2]);
                r.set_ma_2(mac_addr[3]);
            })
            .await?;
        self.dev
            .marl()
            .write_async(|r| {
                r.set_ma_1(mac_addr[4]);
                r.set_ma_0(mac_addr[5]);
            })
            .await?;
        Ok(())
    }

    /// Retrieve the MAC address from the chip.
    ///
    /// N.B: it doesn't come with one, so at startup this will be zeroed or garbage
    pub async fn get_mac(&mut self) -> Result<[u8; 6], Error> {
        let high = self.dev.marh().read_async().await?;
        let med = self.dev.marm().read_async().await?;
        let low = self.dev.marl().read_async().await?;
        Ok([
            high.ma_5(),
            high.ma_4(),
            med.ma_3(),
            med.ma_2(),
            low.ma_1(),
            low.ma_0(),
        ])
    }

    /// Is the link status good (i.e. up)
    pub async fn link_good(&mut self) -> Result<bool, Error> {
        Ok(self.dev.p_1_mbsr().read_async().await?.link_status())
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
        let available = self.dev.txmir().read_async().await?.txma();
        #[cfg(feature = "defmt")]
        defmt::debug!("TXMIR::txma (tx mem avail) = {}", available);
        if (tx_len + 4) > available.into() {
            // No room in the device's buffer currently
            self.dev
                .txntfsr()
                .write_with_zero_async(|r| r.set_txntfs((tx_len + 4) as u16))
                .await?;
            self.dev
                .txqcr()
                .write_with_zero_async(|r| r.set_txqmam(true))
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
        let ier = self.dev.ier().read_async().await?;
        self.dev.ier().write_with_zero_async(|_| {}).await?;
        // Enable TXQ write access
        self.dev.rxqcr().modify_async(|r| r.set_sda(true)).await?;

        let byte_count: [u8; 2] = (buf.len() as u16).to_le_bytes();

        let mut txc = TxCtrlWord::new_zero();
        txc.set_transmit_interrupt_on_completion(true);
        txc.set_frame_id(self.next_frame_id);

        let _pad = (4 - (buf.len() % 4)) % 4;
        let pad = &mut [0u8; 3][0.._pad];

        self.dev
            .interface
            .bus
            .transaction(&mut [
                Operation::Write(&[(Opcode::TXWrite as u8) << 6]),
                Operation::Write(txc.get_inner_buffer()),
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
        self.dev.rxqcr().modify_async(|r| r.set_sda(false)).await?;

        // Manually enqueue the frame
        self.dev.txqcr().modify_async(|r| r.set_metfe(true)).await?;

        // Reenable interrupts
        self.dev.ier().write_async(|r| *r = ier).await?;

        Ok(())
    }

    // Get the number of RX frames ready to be read from the chip.
    // N.B. only updated on interrupt - if no interrupts are enabled this doesn't change!
    pub async fn rx_frames_available(&mut self) -> Result<u8, Error> {
        Ok(self.dev.rxfctr().read_async().await?.rxfc())
    }

    /// Receive a single frame from the chip.
    pub async fn rx(&mut self, rx_buf: &mut [u8]) -> Result<usize, Error> {
        // Disable interrupts
        let ier = self.dev.ier().read_async().await?;
        assert!(!ier.rxie());
        self.dev.ier().write_with_zero_async(|_| {}).await?;

        let frame_status = self.dev.rxfhsr().read_async().await?;
        let byte_count = self.dev.rxfhbcr().read_async().await?.rxbc();
        #[cfg(feature = "defmt")]
        defmt::debug!("frame RX, {} bytes, {}", byte_count, frame_status);
        if !frame_status.rxfv() {
            // Either there is no frame or it's not done receiving.
            return Err(Error::RxNoFrameAvailable);
        }
        if frame_status.rxce()
            || frame_status.rxrf()
            || frame_status.rxftl()
            || frame_status.rxmr()
            || frame_status.rxudpfcs()
            || frame_status.rxtcpfcs()
            || frame_status.rxipfcs()
            || frame_status.rxicmpfcs()
        {
            // Frame error - discard
            self.dev.rxqcr().modify_async(|r| r.set_rrxef(true)).await?;
            // We need to wait until this is cleared before trying to rx again
            while self.dev.rxqcr().read_async().await?.rrxef() {}
            return Err(Error::RxFrameInvalid);
        }
        if usize::from(byte_count) > rx_buf.len() {
            panic!("RX byte count too big!!!");
        }

        // Reset the rx frame pointer
        self.dev.rxfdpr().modify_async(|r| r.set_rxfp(0)).await?;

        // Enable DMA
        self.dev.rxqcr().modify_async(|r| r.set_sda(true)).await?;

        // We need to read a multiple of 4 bytes in total - so we may need some padding
        let pad = (4 - (byte_count % 4)) % 4;
        let discard = &mut [0u8; 3];

        let mut status = Rxfhsr::new_zero();
        let mut bc = Rxfhbcr::new_zero();

        let crc = &mut [0u8; 4];

        self.dev
            .interface
            .bus
            .transaction(&mut [
                Operation::Write(&[(Opcode::RXRead as u8) << 6]),
                // 4 dummy bytes
                Operation::Read(&mut [0u8; 4]),
                // Two status word bytes
                Operation::Read(status.get_inner_buffer_mut()),
                // Two byte count bytes
                Operation::Read(bc.get_inner_buffer_mut()),
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

        assert_eq!(frame_status, status);
        assert_eq!(byte_count, bc.rxbc());

        // Disable DMA
        self.dev.rxqcr().modify_async(|r| r.set_sda(false)).await?;

        // Reenable interrupts
        self.dev.ier().write_async(|r| *r = ier).await?;

        Ok((byte_count - 4).into())
    }
}
