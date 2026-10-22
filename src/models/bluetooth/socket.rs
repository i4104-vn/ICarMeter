// Linux raw Bluetooth HCI socket structures and constants

pub const BTPROTO_HCI: libc::c_int = 1;
pub const SOL_HCI: libc::c_int = 0;
pub const HCI_FILTER: libc::c_int = 2;

pub const HCI_EVENT_PKT: u8 = 0x04;
pub const EVT_LE_META_EVENT: u8 = 0x3E;
pub const EVT_LE_ADVERTISING_REPORT: u8 = 0x02;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SockAddrHci {
    pub hci_family: libc::sa_family_t,
    pub hci_dev: u16,
    pub hci_channel: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HciFilter {
    pub type_mask: u32,
    pub event_mask: [u32; 2],
    pub opcode: u16,
}
