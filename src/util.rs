use dnrs::Flags;

pub fn set_response_flags(mut flags: Flags) -> Flags {
    flags.set_qr(true);
    flags.set_aa(false);
    flags.set_ra(true);
    flags
}
