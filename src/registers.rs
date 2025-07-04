use bondrewd::BitfieldEnum;
use defmt::Format;
use embedded_registers::register;

#[derive(BitfieldEnum, Clone, Debug, PartialEq, Eq, Default, Format)]
#[bondrewd_enum(u8)]
enum MDIXStatus {
    #[default]
    Mdix = 0,
    Mdi = 1,
}

#[derive(BitfieldEnum, Clone, Debug, PartialEq, Eq, Default, Format)]
#[bondrewd_enum(u8)]
enum LinkSpeed {
    #[default]
    _10 = 0,
    _100 = 1,
}

#[register(address = 0x08, mode = "r")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct CCR {
    #[bondrewd(bit_length = 6)]
    __: u8,
    eeprom_presence: bool,
    spi_host: bool,
    #[bondrewd(bit_length = 7)]
    ___: u8,
    x32_pin_package: bool,
}

#[register(address = 0x10, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct MARL {
    #[bondrewd(endianness = "be")]
    marl: [u8; 2],
}
#[register(address = 0x12, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct MARM {
    #[bondrewd(endianness = "be")]
    marm: [u8; 2],
}
#[register(address = 0x14, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct MARH {
    #[bondrewd(endianness = "be")]
    marh: [u8; 2],
}

#[register(address = 0x24, mode = "r")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct MBIR {
    #[bondrewd(bit_length = 3)]
    __: u8,
    #[bondrewd(bit_length = 1)]
    tx_memory_bist_finish: bool,
    #[bondrewd(bit_length = 1)]
    tx_memory_bist_fail: bool,
    #[bondrewd(bit_length = 3)]
    tx_memory_bist_test_fail_count: u8,
    #[bondrewd(bit_length = 3)]
    ___: u8,
    #[bondrewd(bit_length = 1)]
    rx_memory_bist_finish: bool,
    #[bondrewd(bit_length = 1)]
    rx_memory_bist_fail: bool,
    #[bondrewd(bit_length = 3)]
    rx_memory_bist_test_fail_count: u8,
}

#[register(address = 0x26, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct GRR {
    #[bondrewd(bit_length = 14, endianness = "be")]
    __: u16,
    qmu_module_soft_reset: bool,
    global_soft_reset: bool,
}

#[register(address = 0x70, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct TXCR {
    #[bondrewd(bit_length = 7)]
    __: u8,
    checksum_gen_icmp: bool,
    ___: bool,
    checksum_gen_tcp: bool,
    checksum_gen_ip: bool,
    flush_transmit_queue: bool,
    flow_control_enable: bool,
    padding_enable: bool,
    crc_enable: bool,
    transmit_enable: bool,
}

#[register(address = 0x74, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXCR1 {
    flush_receive_queue: bool,
    receive_udp_frame_checksum_check_enable: bool,
    receive_tcp_frame_checksum_check_enable: bool,
    receive_ip_frame_checksum_check_enable: bool,
    receive_physical_address_filtering_with_mac_address_enable: bool,
    receive_flow_control_enable: bool,
    receive_error_frame_enable: bool,
    receive_multicast_address_filtering_with_mac_address_enable: bool,
    receive_broadcast_enable: bool,
    receive_multicast_enable: bool,
    receive_unicast_enable: bool,
    receive_all_enable: bool,
    #[bondrewd(bit_length = 2)]
    __: u8,
    receive_inverse_filtering: bool,
    receive_enable: bool,
}

#[derive(BitfieldEnum, Clone, Debug, PartialEq, Eq, Default, Format)]
#[bondrewd_enum(u8)]
pub enum SPIRxDataBurstLength {
    #[default]
    _4BYTE = 0,
    _8BYTE = 1,
    _16BYTE = 2,
    _32BYTE = 3,
    SINGLEFRAME = 4,
}

#[register(address = 0x76, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXCR2 {
    __: u8,
    #[bondrewd(enum_primitive = "u8", bit_length = 3)]
    spi_receive_data_burst_length: SPIRxDataBurstLength,
    ip4_ip6_udp_fragment_frame_pass: bool,
    receive_ip4_ip6_udp_frame_checksum_equal_zero: bool,
    udp_lite_frame_enable: bool,
    receive_icmp_frame_checksum_check_enable: bool,
    receive_source_address_filtering: bool,
}

#[register(address = 0x78, mode = "r")]
#[bondrewd(reverse, default_endianness = "be", enforce_bytes = 2)]
pub struct TXMIR {
    #[bondrewd(bit_length = 3)]
    __: u8,
    #[bondrewd(bit_length = 13)]
    txma_memory_available: u16,
}

#[register(address = 0x7C, mode = "r")]
#[bondrewd(reverse, default_endianness = "be", enforce_bytes = 2)]
pub struct RXFHSR {
    pub frame_valid: bool,
    __: bool,
    pub icmp_checksum_status: bool,
    pub ip_checksum_status: bool,
    pub tcp_checksum_status: bool,
    pub udp_checksum_status: bool,
    #[bondrewd(bit_length = 2)]
    ___: u8,
    pub broadcast_frame: bool,
    pub multicast_frame: bool,
    pub unicast_frame: bool,
    pub mii_error: bool,
    pub frame_type: bool,
    pub frame_too_long: bool,
    pub runt_frame: bool,
    pub crc_error: bool,
}

#[register(address = 0x7E, mode = "r")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXFHBCR {
    #[bondrewd(bit_length = 4)]
    __: u8,
    #[bondrewd(bit_length = 12, endianness = "be")]
    receive_byte_count: u16,
}

#[register(address = 0x80, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct TXQCR {
    #[bondrewd(bit_length = 13, endianness = "be")]
    __: u16,
    auto_enqueue_txq_frame_enable: bool,
    txq_memory_available_monitor: bool,
    manual_enqueue_txq_frame_enable: bool,
}

#[register(address = 0x82, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXQCR {
    #[bondrewd(bit_length = 3)]
    __: u8,
    rx_duration_timer_threshold_status: bool,
    rx_data_byte_count_threshold_status: bool,
    rx_frame_count_threshold_status: bool,
    rx_ip_header_two_byte_offset_enable: bool,
    ___: bool,
    rx_duration_timer_threshold_enable: bool,
    rx_data_byte_count_threshold_enable: bool,
    rx_frame_count_threshold_enable: bool,
    auto_dequeue_rxq_frame_enable: bool,
    start_dma_access: bool,
    #[bondrewd(bit_length = 2)]
    ____: u8,
    release_rx_error_frame: bool,
}

#[register(address = 0x84, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct TXFDPR {
    __: bool,
    tx_frame_data_pointer_auto_increment: bool,
    #[bondrewd(bit_length = 3)]
    ___: u8,
    #[bondrewd(bit_length = 11, endianness = "be")]
    tx_frame_pointer: u16,
}

#[register(address = 0x86, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXFDPR {
    __: bool,
    rx_frame_pointer_auto_increment: bool,
    #[bondrewd(bit_length = 3)]
    ___: u8,
    #[bondrewd(bit_length = 11, endianness = "be")]
    rx_frame_pointer: u16,
}

#[register(address = 0x8C, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXDTTR {
    #[bondrewd(endianness = "be")]
    receive_duration_timer_threshold: u16,
}

#[register(address = 0x8E, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXDBCTR {
    #[bondrewd(endianness = "be")]
    receive_data_byte_count_threshold: u16,
}

#[register(address = 0x90, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct IER {
    link_change_enable: bool,
    transmit_enable: bool,
    receive_enable: bool,
    __: bool,
    receive_overrun_enable: bool,
    ___: bool,
    transmit_process_stopped_enable: bool,
    receive_process_stopped_enable: bool,
    ____: bool,
    transmit_space_available_enable: bool,
    receive_wakeup_frame_detect_enable: bool,
    receive_magic_packet_detect_enable: bool,
    linkup_detect_enable: bool,
    energy_detect_enable: bool,
    spi_bus_error_enable: bool,
    delay_energy_detect_enable: bool,
}

#[register(address = 0x92, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct ISR {
    /// Link status changed
    pub link_change: bool,
    pub transmit: bool,
    /// Received a packet, ready for us to read it
    pub receive: bool,
    _____: bool,
    pub receive_overrun: bool,
    ____: bool,
    pub transmit_process_stopped: bool,
    pub receive_process_stopped: bool,
    ___: bool,
    pub transmit_space_available: bool,
    pub receive_wakeup_frame_detect: bool, // RO
    pub receive_magic_packet_detect: bool, // RO
    pub linkup_detect: bool,               // RO
    pub energy_detect: bool,               // RO
    pub spi_bus_error: bool,
    __: bool,
}

#[register(address = 0x9C, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct RXFCTR {
    rx_frame_count: u8,
    receive_frame_count_threshold: u8,
}

#[register(address = 0x9E, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct TXNTFSR {
    #[bondrewd(endianness = "be")]
    tx_next_total_frame_size: u16,
}

#[register(address = 0xC0, mode = "r")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct CIDER {
    #[bondrewd(bit_length = 8)]
    family_id: u8,
    #[bondrewd(bit_length = 4)]
    chip_id: u8,
    #[bondrewd(bit_length = 3)]
    revision_id: u8,
    #[bondrewd(bit_length = 1)]
    __: bool,
}

#[register(address = 0xF6, mode = "rw")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct P1CR {
    led_off: bool,
    txids: bool,
    restart_an: bool,
    __: bool,
    ___: bool,
    disable_auto_mdix: bool,
    force_mdix: bool,
    ____: bool,
    auto_negotiation_enable: bool,
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    force_speed: LinkSpeed,
    force_duplex: bool,
    advertise_flow_control_capability: bool,
    advertise_100bt_full_duplex_cabability: bool,
    advertise_100bt_half_duplex_cabability: bool,
    advertise_10bt_full_duplex_cabability: bool,
    advertise_10bt_half_duplex_cabability: bool,
}

#[register(address = 0xF8, mode = "r")]
#[bondrewd(reverse, enforce_bytes = 2)]
pub struct P1SR {
    #[bondrewd(bit_length = 1)]
    hp_mdix: bool,

    #[bondrewd(bit_length = 1)]
    __: bool,

    #[bondrewd(bit_length = 1)]
    polarity_reversed: bool,

    #[bondrewd(bit_length = 2)]
    ___: u8,

    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    speed: LinkSpeed,

    #[bondrewd(bit_length = 1)]
    full_duplex: bool,

    #[bondrewd(bit_length = 1)]
    ____: u8,

    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    mdix_status: MDIXStatus,

    #[bondrewd(bit_length = 1)]
    autonegotiation_done: bool,

    #[bondrewd(bit_length = 1)]
    link_good: bool,

    #[bondrewd(bit_length = 1)]
    partner_flow_control_capable: bool,

    #[bondrewd(bit_length = 1)]
    partner_100bt_full_duplex_capable: bool,

    #[bondrewd(bit_length = 1)]
    partner_100bt_half_duplex_capable: bool,

    #[bondrewd(bit_length = 1)]
    partner_10bt_full_duplex_capable: bool,

    #[bondrewd(bit_length = 1)]
    partner_10bt_half_duplex_capable: bool,
}
