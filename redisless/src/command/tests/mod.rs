use crate::command::Command;
use crate::protocol::Resp;

#[test]
fn set_command() {
    let commands = vec![b"SET", b"set"];
    for cmd in commands {
        let resp = vec![
            Resp::BulkString(cmd),
            Resp::BulkString(b"mykey"),
            Resp::BulkString(b"value"),
        ];

        let command = Command::parse(resp).unwrap();
        dbg!(&command);
        assert_eq!(command, Command::Set(b"mykey".to_vec(), b"value".to_vec()));
    }
}

#[test]
fn info_command() {
    let commands = vec![b"INFO", b"info"];
    for cmd in commands {
        let resp = vec![Resp::BulkString(cmd)];

        let command = Command::parse(resp).unwrap();
        dbg!(command);
    }
}
