use dnrs::dns::{Flags, Header, Networkable, Question, RecordType};

pub fn create_query(domain_name: &str, record_type: RecordType) -> Result<Vec<u8>, ()> {
    let id = rand::random::<u16>();

    let mut flags = Flags::new();
    flags.set_rd(true);

    let header = Header {
        id,
        flags,
        qd_count: 1,
        ..Default::default()
    };

    let question = Question::new(domain_name, record_type)?;

    let mut ret = Vec::new();
    ret.append(&mut header.to_bytes());
    ret.append(&mut question.to_bytes());

    Ok(ret)
}
