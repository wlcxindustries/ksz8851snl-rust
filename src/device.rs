use embedded_hal::spi::Operation;

use crate::driver::{Opcode, reg_cmd};

device_driver::create_device!(
    device_name: Ksz8851snl,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type CommandAddressType = u8;
            type BufferAddressType = u8;
            type DefaultFieldAccess = RO;
            type DefaultByteOrder = LE;
            type DefaultBitOrder = LSB0;
        }
        /// Chip Configuration Register
        register CCR {
            const ADDRESS = 0x08;
            const SIZE_BITS = 16;

            eeprom_presence: bool = 9,
            spi_bus_mode: bool = 8,
            x32_pin_package: bool = 0,
        },
        /// Host MAC Address Register Low
        register MARL {
            const ADDRESS = 0x10;
            const SIZE_BITS = 16;
            ma0: RW uint = 0..=7,
            ma1: RW uint = 8..=15,
        },
        /// Host MAC Address Register Middle
        register MARM {
            const ADDRESS = 0x12;
            const SIZE_BITS = 16;
            ma2: RW uint = 0..=7,
            ma3: RW uint = 8..=15,
        },
        /// Host MAC Address Register High
        register MARH {
            const ADDRESS = 0x14;
            const SIZE_BITS = 16;
            ma4: RW uint = 0..=7,
            ma5: RW uint = 8..=15,
        },
        /// On-chip Bus Control Register
        register OBCR {
            const ADDRESS = 0x20;
            const SIZE_BITS = 16;

            output_pin_drive_strength: RW uint as enum OutputPinDriveStrength {
                x8mA = 0,
                x16mA = 1,
            } = 6..=6,
            on_chip_bus_clock_selection: RW bool = 2,
            on_chip_bus_clock_divider_selection: RW uint as enum OCBCDS {
                x1 = 0,
                x2 = 1,
                x3 = 2,
                Reserved = 3,
            } = 0..=1,
        },
        /// EEPROM Control Register
        register EEPCR {
            const ADDRESS = 0x22;
            const SIZE_BITS = 16;

            /// EEPROM Software Read or Write Access
            eesrwa: WO bool = 5,
            /// EEPROM Software Access
            eesa: RW bool = 4,
            /// EEPROM Status Bit
            eesb: RO bool = 3,
            /// EEPROM Control Bits: Data Transmit
            eecb_data_transmit: RW bool = 2,
            /// EEPROM Control Bits: Serial Clock
            eecb_serial_clock: RW bool = 1,
            /// EEPROM Control Bits: Chip Select
            eecb_chip_select: RW bool = 0,
        },
        /// Memory BIST Info Register
        register MBIR {
            const ADDRESS = 0x24;
            const SIZE_BITS = 16;

            /// TX Memory BIST Test Finish
            /// When set, it indicates the Memory Built In Self Test completion for the TX Memory.
            txmbf: bool = 12,
            /// TX Memory BIST Test Fail
            /// When set, it indicates the TX Memory Built In Self Test has failed
            txmbfa: bool = 11,
            /// TX Memory BIST Test Fail Count
            txmbfc: uint = 8..=10,

            /// RX Memory BIST Test Finish
            /// When set, it indicates the Memory Built In Self Test completion for the RX Memory.
            rxmbf: bool = 4,
            /// RX Memory BIST Test Fail
            /// When set, it indicates the RX Memory Built In Self Test has failed
            rxmbfa: bool = 3,
            /// RX Memory BIST Test Fail Count
            rxmbfc: uint = 0..=2,
        },
        /// Global Reset Register
        register GRR {
            const ADDRESS = 0x26;
            const SIZE_BITS = 16;
            /// QMU Module Soft Reset
            /// 1: Software reset is active to clear both TXQ and RXQ memories.
            /// 0: Software reset is inactive.
            /// QMU software reset will flush out all TX/RX packet data inside the TXQ
            /// and RXQ memories and reset all QMU registers to default value.
            qmu_module_soft_reset: RW bool = 1,
            /// Global Soft Reset
            /// 1: Software reset is active.
            /// 0: Software reset is inactive.
            /// Global software reset will affect PHY, MAC, QMU, DMA, and the switch
            /// core, all registers value are set to default value.
            global_soft_reset: RW bool = 0,
        },
        /// Wakeup Frame Control Register
        register WFCR {
            const ADDRESS = 0x2A;
            const SIZE_BITS = 16;

            /// Magic Packet RX Enable
            /// When set, it enables the magic packet pattern detection.
            /// When reset, the magic packet pattern detection is disabled.
            mprxe: RW bool = 7,
            /// Wake up Frame 3 Enable
            /// When set, it enables the Wake up frame 3 pattern detection.
            /// When reset, the Wake up frame 3 pattern detection is disabled.
            wf3e: RW bool = 3,
            /// Wake up Frame 2 Enable
            /// When set, it enables the Wake up frame 2 pattern detection.
            /// When reset, the Wake up frame 2 pattern detection is disabled.
            wf2e: RW bool = 2,
            /// Wake up Frame 1 Enable
            /// When set, it enables the Wake up frame 1 pattern detection.
            /// When reset, the Wake up frame 1 pattern detection is disabled.
            wf1e: RW bool = 1,
            /// Wake up Frame 0 Enable
            /// When set, it enables the Wake up frame 0 pattern detection.
            /// When reset, the Wake up frame 0 pattern detection is disabled.
            wf0e: RW bool = 0,
        },

        // Wakeup frame 0

        /// Wakeup Frame 0 CRC0 Register
        register WF0CRC0 {
            const ADDRESS = 0x30;
            const SIZE_BITS = 16;
            /// Wake up Frame 0 CRC (lower 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf0crc0: RW uint = 0..=15,
        },
        /// Wakeup Frame 0 CRC1 Register
        register WF0CRC1 {
            const ADDRESS = 0x32;
            const SIZE_BITS = 16;
            /// Wake up Frame 0 CRC (upper 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf0crc1: RW uint = 0..=15,
        },
        /// Wakeup Frame 0 Byte Mask 0 Register
        register WF0BM0 {
            const ADDRESS = 0x34;
            const SIZE_BITS = 16;

            /// Wake up Frame 0 Byte Mask 0
            /// The first 16 bytes mask of a Wake up frame 0 pattern.
            wf0bm0: RW uint = 0..=15,
        },
        /// Wakeup Frame 0 Byte Mask 1 Register
        register WF0BM1 {
            const ADDRESS = 0x36;
            const SIZE_BITS = 16;

            /// Wake up Frame 0 Byte Mask 1
            /// The next 16 bytes mask covering bytes 17 to 32 of a Wake up frame 0 pattern.
            wf0bm1: RW uint = 0..=15,
        },
        /// Wakeup Frame 0 Byte Mask 2 Register
        register WF0BM2 {
            const ADDRESS = 0x38;
            const SIZE_BITS = 16;

            /// Wake up Frame 0 Byte Mask 2
            /// The next 16 bytes mask covering bytes 33 to 48 of a Wake-up frame 0 pattern.
            wf0bm2: RW uint = 0..=15,
        },
        /// Wakeup Frame 0 Byte Mask 3 Register
        register WF0BM3 {
            const ADDRESS = 0x3A;
            const SIZE_BITS = 16;

            /// Wake up Frame 0 Byte Mask 3
            /// The last 16 bytes mask covering bytes 49 to 64 of a Wake-up frame 0 pattern.
            wf0bm2: RW uint = 0..=15,
        },

        // Wakeup frame 1

        /// Wakeup Frame 1 CRC0 Register
        register WF1CRC0 {
            const ADDRESS = 0x40;
            const SIZE_BITS = 16;
            /// Wake up Frame 1 CRC (lower 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf1crc0: RW uint = 0..=15,
        },
        /// Wakeup Frame 1 CRC1 Register
        register WF1CRC1 {
            const ADDRESS = 0x42;
            const SIZE_BITS = 16;
            /// Wake up Frame 1 CRC (upper 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf1crc1: RW uint = 0..=15,
        },
        /// Wakeup Frame 1 Byte Mask 0 Register
        register WF1BM0 {
            const ADDRESS = 0x44;
            const SIZE_BITS = 16;

            /// Wake up Frame 1 Byte Mask 0
            /// The first 16 bytes mask of a Wake up frame 0 pattern.
            wf1bm0: RW uint = 0..=15,
        },
        /// Wakeup Frame 1 Byte Mask 1 Register
        register WF1BM1 {
            const ADDRESS = 0x46;
            const SIZE_BITS = 16;

            /// Wake up Frame 1 Byte Mask 1
            /// The next 16 bytes mask covering bytes 17 to 32 of a Wake up frame 0 pattern.
            wf1bm1: RW uint = 0..=15,
        },
        /// Wakeup Frame 1 Byte Mask 2 Register
        register WF1BM2 {
            const ADDRESS = 0x48;
            const SIZE_BITS = 16;

            /// Wake up Frame 1 Byte Mask 2
            /// The next 16 bytes mask covering bytes 33 to 48 of a Wake-up frame 0 pattern.
            wf1bm2: RW uint = 0..=15,
        },
        /// Wakeup Frame 1 Byte Mask 3 Register
        register WF1BM3 {
            const ADDRESS = 0x4A;
            const SIZE_BITS = 16;

            /// Wake up Frame 1 Byte Mask 3
            /// The last 16 bytes mask covering bytes 49 to 64 of a Wake-up frame 0 pattern.
            wf1bm2: RW uint = 0..=15,
        },

        // Wakeup frame 2

        /// Wakeup Frame 2 CRC0 Register
        register WF2CRC0 {
            const ADDRESS = 0x50;
            const SIZE_BITS = 16;
            /// Wake up Frame 2 CRC (lower 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf2crc0: RW uint = 0..=15,
        },
        /// Wakeup Frame 2 CRC1 Register
        register WF2CRC1 {
            const ADDRESS = 0x52;
            const SIZE_BITS = 16;
            /// Wake up Frame 2 CRC (upper 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf2crc1: RW uint = 0..=15,
        },
        /// Wakeup Frame 2 Byte Mask 0 Register
        register WF2BM0 {
            const ADDRESS = 0x54;
            const SIZE_BITS = 16;

            /// Wake up Frame 2 Byte Mask 0
            /// The first 16 bytes mask of a Wake up frame 0 pattern.
            wf2bm0: RW uint = 0..=15,
        },
        /// Wakeup Frame 2 Byte Mask 1 Register
        register WF2BM1 {
            const ADDRESS = 0x56;
            const SIZE_BITS = 16;

            /// Wake up Frame 2 Byte Mask 1
            /// The next 16 bytes mask covering bytes 17 to 32 of a Wake up frame 0 pattern.
            wf2bm1: RW uint = 0..=15,
        },
        /// Wakeup Frame 2 Byte Mask 2 Register
        register WF2BM2 {
            const ADDRESS = 0x58;
            const SIZE_BITS = 16;

            /// Wake up Frame 2 Byte Mask 2
            /// The next 16 bytes mask covering bytes 33 to 48 of a Wake-up frame 0 pattern.
            wf2bm2: RW uint = 0..=15,
        },
        /// Wakeup Frame 2 Byte Mask 3 Register
        register WF2BM3 {
            const ADDRESS = 0x5A;
            const SIZE_BITS = 16;

            /// Wake up Frame 2 Byte Mask 3
            /// The last 16 bytes mask covering bytes 49 to 64 of a Wake-up frame 0 pattern.
            wf2bm2: RW uint = 0..=15,
        },

        // Wakeup frame 3

        /// Wakeup Frame 3 CRC0 Register
        register WF3CRC0 {
            const ADDRESS = 0x60;
            const SIZE_BITS = 16;
            /// Wake up Frame 3 CRC (lower 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf3crc0: RW uint = 0..=15,
        },
        /// Wakeup Frame 3 CRC1 Register
        register WF3CRC1 {
            const ADDRESS = 0x62;
            const SIZE_BITS = 16;
            /// Wake up Frame 3 CRC (upper 16 bits)
            /// The expected CRC value of a Wake up frame 0 pattern
            wf3crc1: RW uint = 0..=15,
        },
        /// Wakeup Frame 3 Byte Mask 0 Register
        register WF3BM0 {
            const ADDRESS = 0x64;
            const SIZE_BITS = 16;

            /// Wake up Frame 3 Byte Mask 0
            /// The first 16 bytes mask of a Wake up frame 0 pattern.
            wf3bm0: RW uint = 0..=15,
        },
        /// Wakeup Frame 3 Byte Mask 1 Register
        register WF3BM1 {
            const ADDRESS = 0x66;
            const SIZE_BITS = 16;

            /// Wake up Frame 3 Byte Mask 1
            /// The next 16 bytes mask covering bytes 17 to 32 of a Wake up frame 0 pattern.
            wf3bm1: RW uint = 0..=15,
        },
        /// Wakeup Frame 3 Byte Mask 2 Register
        register WF3BM2 {
            const ADDRESS = 0x68;
            const SIZE_BITS = 16;

            /// Wake up Frame 3 Byte Mask 2
            /// The next 16 bytes mask covering bytes 33 to 48 of a Wake-up frame 0 pattern.
            wf3bm2: RW uint = 0..=15,
        },
        /// Wakeup Frame 3 Byte Mask 3 Register
        register WF3BM3 {
            const ADDRESS = 0x6A;
            const SIZE_BITS = 16;

            /// Wake up Frame 3 Byte Mask 3
            /// The last 16 bytes mask covering bytes 49 to 64 of a Wake-up frame 0 pattern.
            wf3bm2: RW uint = 0..=15,
        },

        /// Transmit Control Register
        register TXCR {
            const ADDRESS = 0x70;
            const SIZE_BITS = 16;

            /// Transmit Checksum Generation for ICMP
            /// When this bit is set, The KSZ8851SNL is enabled to transmit ICMP frame (only
            /// for non-fragment frame) checksum generation.
            tcgicmp: RW bool = 8,
            /// Transmit Checksum Generation for TCP
            /// When this bit is set, The KSZ8851SNL is enabled to transmit TCP frame
            /// checksum generation.
            tcgtcp: RW bool = 6,
            /// Transmit Checksum Generation for IP
            /// When this bit is set, The KSZ8851SNL is enabled to transmit IP header
            /// checksum generation.
            tcgip: RW bool = 5,
            /// Flush Transmit Queue
            /// When this bit is set, The transmit queue memory is cleared and TX
            /// frame pointer is reset.
            /// Note: Disable the TXE transmit enable bit[0] first before set this bit, then
            /// clear this bit to normal operation.
            ftxq: RW bool = 4,
            /// Transmit Flow Control Enable
            /// When this bit is set and the KSZ8851SNL is in full-duplex mode, flow
            /// control is enabled. The KSZ8851SNL transmits a PAUSE frame when
            /// the Receive Buffer capacity reaches a threshold level that will cause the
            /// buffer to overflow.
            /// When this bit is set and the KSZ8851SNL is in half-duplex mode, back-
            /// pressure flow control is enabled. When this bit is cleared, no transmit
            /// flow control is enabled.
            txfce: RW bool = 3,
            /// Transmit Padding Enable
            /// When this bit is set, the KSZ8851SNL automatically adds a padding field
            /// to a packet shorter than 64 bytes.
            /// Note: Setting this bit requires enabling the add CRC feature (bit1=1) to
            /// avoid CRC errors for the transmit packet.
            txpe: RW bool = 2,
            /// Transmit CRC Enable
            /// When this bit is set, the KSZ8851SNL automatically adds a 32-bit CRC
            /// checksum field to the end of a transmit frame.
            txce: RW bool = 1,
            /// Transmit Enable
            /// When this bit is set, the transmit module is enabled and placed in a running
            /// state. When reset, the transmit process is placed in the stopped
            /// state after the transmission of the current frame is completed.
            txe: RW bool = 0,
        },

        /// Transmit Status Register
        register TXSR {
            const ADDRESS = 0x72;
            const SIZE_BITS = 16;

            /// Transmit Late Collision
            /// This bit is set when a transmit late collision occurs
            txlc: bool = 13,
            /// Transmit Maximum Collision
            /// This bit is set when a transmit Maximum Collision is reached.
            txmc: bool = 12,
            /// Transmit Frame ID
            /// This field identifies the transmitted frame. All of the transmit status
            /// information in this register belongs to the frame with this ID.
            txfid: uint = 0..=5,
        },

        /// Receive Control Register 1
        register RXCR1 {
            const ADDRESS = 0x74;
            const SIZE_BITS = 16;

            /// Flush Receive Queue
            /// When this bit is set, The receive queue memory is cleared and RX frame
            /// pointer is reset.
            /// Note: Disable the RXE receive enable bit[0] first before set this bit, then
            /// clear this bit to normal operation.
            frxq: RW bool = 15,
            /// Receive UDP Frame Checksum Check Enable
            /// When this bit is set, the KSZ8851SNL will check for correct UDP check-
            /// sum for incoming UDP frames. Any received UDP frames with incorrect
            /// checksum will be discarded.
            rxudpfcc: RW bool = 14,
            ///  Receive TCP Frame Checksum Check Enable
            /// When this bit is set, the KSZ8851SNL will check for correct TCP check-
            /// sum for incoming TCP frames. Any received TCP frames with incorrect
            /// checksum will be discarded.
            rxtcpfcc: RW bool = 13,
            /// Receive IP Frame Checksum Check Enable
            /// When this bit is set, the KSZ8851SNL will check for correct IP header
            /// checksum for incoming IP frames. Any received IP frames with incorrect
            /// checksum will be discarded.
            rxipfcc: RW bool = 12,
            /// Receive Physical Address Filtering with MAC Address Enable
            /// When this bit is set, this bit enables the RX function to receive physical
            /// address that pass the MAC address filtering mechanism (see Address
            /// Filtering Scheme table for detail).
            rxpafma: RW bool = 11,
            /// Receive Flow Control Enable
            /// When this bit is set and the KSZ8851SNL is in full-duplex mode, flow
            /// control is enabled, and the KSZ8851SNL will acknowledge a PAUSE
            /// frame from the receive interface; i.e., the outgoing packets are pending
            /// in the transmit buffer until the PAUSE frame control timer expires. This
            /// field has no meaning in half-duplex mode and should be programmed to
            /// 0. When this bit is cleared, flow control is not enabled.
            rxfce: RW bool = 10,
            /// Receive Error Frame Enable
            /// When this bit is set, CRC error frames are allowed to be received into
            /// the RX queue.
            /// When this bit is cleared, all CRC error frames are discarded.
            rxefe: RW bool = 9,
            /// Receive Multicast Address Filtering with MAC Address Enable
            /// When this bit is set, this bit enables the RX function to receive multicast
            /// address that pass the MAC address filtering mechanism (see Address
            /// Filtering Scheme table for detail).
            rxmafma: RW bool = 8,
            /// Receive Broadcast Enable
            /// When this bit is set, the RX module receives all the broadcast frames.
            rxbe: RW bool = 7,
            /// Receive Multicast Enable
            /// When this bit is set, the RX module receives all the multicast frames
            /// (including broadcast frames).
            rxme: RW bool = 6,
            /// Receive Unicast Enable
            /// When this bit is set, the RX module receives unicast frames that match
            /// the 48-bit Station MAC address of the module.
            rxue: RW bool = 5,
            /// Receive All Enable
            /// When this bit is set, the KSZ8851SNL receives all incoming frames,
            /// regardless of the frame’s destination address (see Address Filtering
            /// Scheme table for detail).
            rxae: RW bool = 4,
            /// Receive Inverse Filtering
            /// When this bit is set, the KSZ8851SNL receives function with address
            /// check operation in inverse filtering mode (see Address Filtering Scheme
            /// table for detail)
            rxinvf: RW bool = 1,
            /// Receive Enable
            /// When this bit is set, the RX block is enabled and placed in a running state.
            /// When this bit is cleared, the receive process is placed in the stopped
            /// state upon completing reception of the current frame.
            rxe: RW bool = 0,
        },

        /// Receive Control Register 2
        register RXCR2 {
            const ADDRESS = 0x76;
            const SIZE_BITS = 16;

            /// SPI Receive Data Burst Length
            /// These three bits are used to define for SPI receive data burst length
            /// during DMA operation from the host CPU to access RXQ frame buffer.
            /// 000: 4 Bytes data burst 001: 8 Bytes data burst
            /// 010: 16 Bytes data burst 011: 32 Bytes data burst
            /// 100: Single frame data burst 101-111: NA (reserved)
            /// Note: It needs RXQ FIFO Read command byte before each data burst.
            srdbl: WO uint as enum SPIRxDataBurstLength {
                x4Bytes = 0,
                x8Bytes = 1,
                x16Bytes = 2,
                x32Bytes = 3,
                SingleFrame = 4,
                Reserved = catch_all,
            } = 5..=7,
            /// IPv4/IPv6/UDP Fragment Frame Pass
            /// When this bit is set, the KSZ8851SNL will pass the checksum check at
            /// receive side for IPv4/IPv6 UDP frame with fragment extension header.
            /// When this bit is cleared, the KSZ8851SNL will perform checksum opera-
            /// tion based on configuration and doesn’t care whether it’s a fragment
            /// frame or not.
            iufpp: RW bool = 4,
            /// Receive IPv4/IPv6/UDP Frame Checksum Equal Zero
            /// When this bit is set, the KSZ8851SNL will pass the filtering for IPv4/IPv6
            /// UDP frame with UDP checksum equal to zero.
            /// When this bit is cleared, the KSZ8851SNL will drop IPv4/IPv6 UDP
            /// packet with UDP checksum equal to zero.
            rxiufcez: RW bool = 3,
            /// UDP Lite Frame Enable
            /// When this bit is set, the KSZ8851SNL will check the checksum at
            /// receive side and generate the checksum at transmit side for UDP Lite
            /// frame.
            /// When this bit is cleared, the KSZ8851SNL will pass the checksum check
            /// at receive side and skip the checksum generation at transmit side for
            /// UDP Lite frame.
            udplfe: RW bool = 2,
            /// Receive ICMP Frame Checksum Check Enable
            /// When this bit is set, the KSZ8851SNL will check for correct ICMP check-
            /// sum for incoming ICMP frames (only for non-fragment frame). Any
            /// received ICMP frames with incorrect checksum will be discarded.
            rxicmpfcc: RW bool = 1,
            /// Receive Source Address Filtering
            /// When this bit is set, the KSZ8851SNL will drop the frame if the source
            /// address is same as MAC address in MARL, MARM, MARH registers.
            rxsaf: RW bool = 0,
        },

        /// TXQ Memory Information Register
        register TXMIR {
            const ADDRESS = 0x78;
            const SIZE_BITS = 16;

            txma: uint = 0..=12,
        },

        /// Receive Frame Header Status Register
        register RXFHSR {
            const ADDRESS = 0x7C;
            const SIZE_BITS = 16;

            /// Receive Frame Valid
            /// When this bit is set, it indicates that the present frame in the receive
            /// packet memory is valid. The status information currently in this location
            /// is also valid.
            /// When clear, it indicates that there is either no pending receive frame or
            /// that the current frame is still in the process of receiving.
            rxfv: bool = 15,
            /// Receive ICMP Frame Checksum Status
            /// When this bit is set, the KSZ8851SNL received ICMP frame checksum
            /// field is incorrect.
            rxicmpfcs: bool = 13,
            /// Receive IP Frame Checksum Status
            /// When this bit is set, the KSZ8851SNL received IP header checksum
            /// field is incorrect.
            rxipfcs: bool = 12,
            // Receive TCP Frame Checksum Status
            // When this bit is set, the KSZ8851SNL received TCP frame checksum
            // field is incorrect
            rxtcpfcs: bool = 11,
            /// Receive UDP Frame Checksum Status
            /// When this bit is set, the KSZ8851SNL received UDP frame checksum
            /// field is incorrect.
            rxudpfcs: bool = 10,
            /// Receive Broadcast Frame
            /// When this bit is set, it indicates that this frame has a broadcast address.
            rxbf: bool = 7,
            /// Receive Multicast Frame
            /// When this bit is set, it indicates that this frame has a multicast address
            /// (including the broadcast address)
            rxmf: bool = 6,
            /// Receive Unicast Frame
            /// When this bit is set, it indicates that this frame has a unicast address.
            rxuf: bool = 5,
            /// Receive MII Error
            /// When set, it indicates that there is an MII symbol error on the received
            /// frame.
            rxmr: bool = 4,
            /// Receive Frame Type
            /// When this bit is set, it indicates that the frame is an Ethernet-type frame
            /// (frame length is greater than 1500 bytes). When clear, it indicates that
            /// the frame is an IEEE 802.3 frame.
            /// This bit is not valid for runt frames.
            rxft: bool = 3,
            /// Receive Frame Too Long
            /// When this bit is set, it indicates that the frame length exceeds the maxi-
            /// mum size of 2000 bytes. Frames that are too long are passed to the host
            /// only if the pass bad frame bit is set.
            /// Note: Frame too long is only a frame length indication and does not
            /// cause any frame truncation
            rxftl: bool = 2,
            /// Receive Runt Frame
            /// When this bit is set, it indicates that a frame was damaged by a collision
            /// or had a premature termination before the collision window passed.
            /// Runt frames are passed to the host only if the pass bad frame bit is set.
            rxrf: bool = 1,
            /// Receive CRC Error
            /// When this bit is set, it indicates that a CRC error has occurred on the
            /// current received frame.
            /// CRC error frames are passed to the host only if the pass bad frame bit is
            /// set.
            rxce: bool = 0,
        },

        /// Receive Frame Header Byte Count Register
        register RXFHBCR {
            const ADDRESS = 0x7E;
            const SIZE_BITS = 16;

            rxbc: uint = 0..=11,
        },

        /// TXQ Command Register
        register TXQCR {
            const ADDRESS = 0x80;
            const SIZE_BITS = 16;

            /// Auto-Enqueue TXQ Frame Enable
            /// When this bit is written as 1, the KSZ8851SNL will enable current all TX
            /// frames prepared in the TX buffer are queued to transmit automatically.
            /// The bit 0 METFE has to be set 0 when this bit is set to 1 in this register.
            aetfe: RW bool = 2,
            /// TXQ Memory Available Monitor
            /// When this bit is written as 1, the KSZ8851SNL will generate interrupt (bit
            /// 6 in ISR register) to CPU when TXQ memory is available based upon
            /// the total amount of TXQ space requested by CPU at TXNTFSR (0x9E)
            /// register.
            /// Note: This bit is self-clearing after the frame is finished transmitting. The
            /// software should wait for the bit to be cleared before set to 1 again.
            txqmam: RW bool = 1,
            /// Manual Enqueue TXQ Frame Enable
            /// When this bit is written as 1, the KSZ8851SNL will enable current TX
            /// frame prepared in the TX buffer is queued for transmit, this is only trans-
            /// mit one frame at a time.
            /// Note: This bit is self-clearing after the frame is finished transmitting. The
            /// software should wait for the bit to be cleared before setting up another
            /// new TX frame.
            metfe: RW bool = 0,
        },

        /// RXQ Command Register
        register RXQCR {
            const ADDRESS = 0x82;
            const SIZE_BITS = 16;

            /// RX Duration Timer Threshold Status
            rxdtts: bool = 12,
            /// RX Data Byte Count Threshold Status
            rxdbcts: bool = 11,
            /// RX Frame Count Threshold Status
            rxfcts: bool = 10,
            /// RX IP Header Two-Byte Offset Enable
            rxiphtoe: RW bool = 9,
            /// RX Duration Timer Threshold Enable
            rxdtte: RW bool = 7,
            /// RX Data Byte Count Threshold Enable
            rxdbcte: RW bool = 6,
            /// RX Frame Count Threshold Enable
            rxfcte: RW bool = 5,
            /// Auto-Dequeue RXQ Frame Enable
            adrfe: RW bool = 4,
            /// Start DMA Access
            sda: WO bool = 3,
            /// Release RX Error Frame
            rrxef: RW bool = 0,
        },

        // TX Frame Data Pointer Register
        register TXFDPR {
            const ADDRESS = 0x84;
            const SIZE_BITS = 16;

            /// TX Frame Data Pointer Auto Increment
            txfpai: RW bool = 14,
            /// TX Frame Pointer
            txfp: uint = 0..=10,
        },

        // RX Frame Data Pointer Register
        register RXFDPR {
            const ADDRESS = 0x86;
            const SIZE_BITS = 16;

            /// RX Frame Data Pointer Auto Increment
            rxfpai: RW bool = 14,
            /// RX Frame Pointer
            rxfp: WO uint = 0..=10,
        },

        /// RX Duration timer Threshold Register
        register RXDTTR {
            const ADDRESS = 0x8C;
            const SIZE_BITS = 16;

            /// Receive Duration Timer Threshold
            rxdtt: RW uint = 0..=15,
        },

        /// RX Data Byte Count Threshold Register
        register RXDBCTR {
            const ADDRESS = 0x8E;
            const SIZE_BITS = 16;

            /// Receive Data Byte Count Threshold
            rxdbct: RW uint = 0..=15,
        },

        /// Interrupt Enable Register
        register IER {
            const ADDRESS = 0x90;
            const SIZE_BITS = 16;

            /// Link Change Interrupt Enable
            lcie: RW bool = 15,
            /// Transmit Interrupt Enabl
            txie: RW bool = 14,
            /// Receive Interrupt Enable
            rxie: RW bool = 13,
            /// Receive Overrun Interrupt Enable
            rxoie: RW bool = 11,
            /// Transmit Process Stopped Interrupt Enable
            txpsie: RW bool = 9,
            /// Receive Process Stopped Interrupt Enable
            rxpsie: RW bool = 8,
            /// Transmit Space Available Interrupt Enable
            txsaie: RW bool = 6,
            /// Receive Wake-up Frame Detect Interrupt Enable
            rxwfdie: RW bool = 5,
            /// Receive Magic Packet Detect Interrupt Enable
            rxmpdie: RW bool = 4,
            /// Linkup Detect Interrupt Enable
            ldie: RW bool = 3,
            /// Energy Detect Interrupt Enable
            edie: RW bool = 2,
            /// SPI Bus Error Interrupt Enable
            spibeie: RW bool = 1,
            /// Delay Energy Detect Interrupt Enable
            dedie: RW bool = 0,
        },

        /// Interrupt Status Register
        register ISR {
            const ADDRESS = 0x92;
            const SIZE_BITS = 16;

            /// Link Change Interrupt Status
            lcis: RW bool = 15,
            /// Transmit Interrupt Status
            txis: RW bool = 14,
            /// Receive Interrupt Status
            rxis: RW bool = 13,
            /// Receive Overrun Interrupt Status
            rxois: RW bool = 11,
            /// Transmit Process Stopped Interrupt Status
            txpsis: RW bool = 9,
            /// Receive Process Stopped Interrupt Status
            rxpsis: RW bool = 8,
            /// Transmit Space Available Interrupt Status
            txsais: RW bool = 6,
            /// Receive Wakeup Frame Detect Interrupt Status
            rxwfdis: RW bool = 5,
            /// Receive Magic Packet Detect Interrupt Status
            rxmpdis: RW bool = 4,
            /// Linkup Detect Interrupt Status
            ldis: RW bool = 3,
            /// Energy Detect Interrupt Status
            edis: RW bool = 2,
            /// SPI Bus Error Interrupt Status
            spibeis: RW bool = 1,
        },

        /// RX Frame Count & Threshold Register
        register RXFCTR {
            const ADDRESS = 0x9C;
            const SIZE_BITS = 16;

            /// RX Frame Count
            rxfc: uint = 8..=15,
            /// Receive Frame Count Threshold
            rxfct: RW uint = 0..=7,
        },

        /// Tx Next Total Frames Size Register
        register TXNTFSR {
            const ADDRESS = 0x9E;
            const SIZE_BITS = 16;

            /// TX Next Total Frames Size
            txntfs: RW uint = 0..=15,
        },

        /// MAC Address Hash Table Register 0
        register MAHTR0 {
            const ADDRESS = 0xA0;
            const SIZE_BITS = 16;

            ht0: RW uint = 0..=15,
        },

        /// MAC Address Hash Table Register 1
        register MAHTR1 {
            const ADDRESS = 0xA2;
            const SIZE_BITS = 16;

            ht1: RW uint = 0..=15,
        },

        /// MAC Address Hash Table Register 2
        register MAHTR2 {
            const ADDRESS = 0xA4;
            const SIZE_BITS = 16;

            ht2: RW uint = 0..=15,
        },

        /// MAC Address Hash Table Register 3
        register MAHTR3 {
            const ADDRESS = 0xA6;
            const SIZE_BITS = 16;

            ht3: RW uint = 0..=15,
        },

        /// Flow Control Low Watermark Register
        register FCLWR {
            const ADDRESS = 0xB0;
            const SIZE_BITS = 16;

            /// Flow Control Low Watermark Configuration
            fclwc: RW uint = 0..=11,

        },

        /// Flow Control High Watermark Register
        register FCHWR {
            const ADDRESS = 0xB2;
            const SIZE_BITS = 16;

            /// Flow Control High Watermark Configuration
            fchwc: RW uint = 0..=11,
        },

        /// Flow Control Overrun Watermark Register
        register FCOWR {
            const ADDRESS = 0xB4;
            const SIZE_BITS = 16;

            /// Flow Control Overrun Watermark Configuration
            fclwc: RW uint = 0..=11,
        },

        /// Chip ID and Enable Register
        register CIDER {
            const ADDRESS = 0xC0;
            const SIZE_BITS = 16;

            /// Chip Family ID
            family_id: uint = 8..=15,
            /// Chip ID. 0x7 is KSZ8851SNL
            chip_id: uint = 4..=7,
            /// Revision ID
            revision_id: uint = 1..=3,
        },

        /// Chip Global Control Register
        register CGCR {
            const ADDRESS = 0xC6;
            const SIZE_BITS = 16;

            /// LEDSEL0 selection for LED1 and LED0
            /// |       |      LEDSEL0    |
            /// |       | false    | true |
            /// | LED1  | 100BT    | ACT  |
            /// | LED0  | Link/Act | LINK |
            ledsel0: RW bool = 9,
        },

        // TODO: some missing registers here

        /// PHY 1 MII-Register Basic Control Register
        register P1MBCR {
            const ADDRESS = 0xE4;
            const SIZE_BITS = 16;

            /// Local (far-end) loopback (llb)
            /// 1 = perform local loopback at host
            /// (host SPI Tx -> PHY -> host SPI Rx)
            /// 0 = normal operation
            local_far_end_loopback: RW bool = 14,
            /// Force 100
            /// 1 = force 100 Mbps if AN is disabled (bit 12)
            /// 0 = force 10 Mbps if AN is disabled (bit 12)
            /// Bit is same as Bit 6 in P1CR.
            force_100: RW bool = 13,
            /// AN Enable
            /// 1 = auto-negotiation enabled.
            /// 0 = auto-negotiation disabled.
            /// Bit is same as Bit 7 in P1CR.
            an_enable: RW bool = 12,
            /// Restart AN
            /// 1 = restart auto-negotiation.
            /// 0 = normal operation.
            /// Bit is same as Bit 13 in P1CR.
            restart_an: RW bool = 9,
            /// Force Full-Duplex
            /// 1 = force full-duplex
            /// 0 = force half-duplex.
            /// if AN is disabled (bit 12) or AN is enabled but failed.
            /// Bit is same as Bit 5 in P1CR.
            force_full_duplex: RW bool = 8,
            /// HP_mdix
            /// 1 = HP Auto MDI-X mode.
            /// 0 = Microchip Auto MDI-X mode.
            /// Bit is same as Bit 15 in P1SR.
            hp_mdix: RW bool = 5,
            /// Force MDI-X
            /// 1 = force MDI-X.
            /// 0 = normal operation.
            /// Bit is same as Bit 9 in P1CR.
            force_mdix: RW bool = 4,
            /// Disable MDI-X
            /// 1 = disable auto MDI-X.
            /// 0 = normal operation.
            /// Bit is same as Bit 10 in P1CR.
            disable_mdix: RW bool = 3,
            /// Disable Transmit
            /// 1 = disable transmit.
            /// 0 = normal operation.
            /// Bit is same as Bit 14 in P1CR.
            disable_transmit: RW bool = 1,
            /// Disable LED
            /// 1 = disable all LEDs.
            /// 0 = normal operation.
            /// Bit is same as Bit 15 in P1CR.
            disable_led: RW bool = 0,
        },

        /// PHY 1 MII-Register Basic Status Register
        register P1MBSR {
            const ADDRESS = 0xE6;
            const SIZE_BITS = 16;

            t4_capable: bool = 15,
            x100_full_capable: bool = 14,
            x100_half_capable: bool = 13,
            x10_full_capable: bool = 12,
            x10_half_capable: bool = 11,
            an_complete: bool = 5,
            an_capable: bool = 3,
            link_status: bool = 2,
            extended_capable: bool = 0,
        },

        // TODO: A few others here too

        /// TX Control Word - used during TX FIFO operations
        /// This is not actually a real register! Don't try to read or write. This is defined here because
        /// Device driver doesn't currently allow standalone fieldsets. See https://github.com/diondokter/device-driver/issues/77
        register TxCtrlWord {
            const ADDRESS = 0xff;
            const SIZE_BITS = 16;

            transmit_interrupt_on_completion: WO bool = 15,
            frame_id: WO uint = 0..=5,
        }
    }
);

pub struct Ksz8851snlInterface<BUS> {
    pub bus: BUS,
}

impl<BUS: embedded_hal_async::spi::SpiDevice> device_driver::AsyncRegisterInterface
    for Ksz8851snlInterface<BUS>
{
    type Error = BUS::Error;

    type AddressType = u8;

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        assert!(size_bits == 16);
        self.bus
            .transaction(&mut [
                Operation::Write(&reg_cmd(Opcode::RegRead, address, 2)),
                Operation::Read(data),
            ])
            .await?;
        Ok(())
    }

    async fn write_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        assert!(size_bits != 16);
        self.bus
            .transaction(&mut [
                Operation::Write(&reg_cmd(Opcode::RegWrite, address, 2)),
                Operation::Write(data),
            ])
            .await
    }
}
