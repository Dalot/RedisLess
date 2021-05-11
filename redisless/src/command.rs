use crate::protocol::error::RedisCommandError;
use crate::protocol::Resp;
use storage::in_memory::Expiry;

type Key = Vec<u8>;
type Value = Vec<u8>;
type Items = Vec<(Key, Value)>;

#[derive(Debug, PartialEq)]
pub enum Command {
    Set(Key, Value),
    Setnx(Key, Value),
    Setex(Key, Expiry, Value),
    MSetnx(Items),
    Expire(Key, Expiry),
    PExpire(Key, Expiry),
    Get(Key),
    GetSet(Key, Value),
    Del(Key),
    Incr(Key),
    Exists(Key),
    Info,
    Ping,
    Quit,
}

fn get_bytes_vec(resp: Option<&Resp>) -> Result<Vec<u8>, RedisCommandError> {
    match resp {
        Some(Resp::String(x)) | Some(Resp::BulkString(x)) => Ok(x.to_vec()),
        _ => Err(RedisCommandError::ArgNumber),
    }
}

fn parse_duration(bytes: Vec<u8>) -> Result<u64, RedisCommandError> {
    let duration = std::str::from_utf8(&bytes[..])?;
    Ok(duration.parse::<u64>()?)
}

impl Command {
    pub fn parse(v: Vec<Resp>) -> Result<Self, RedisCommandError> {
        use Command::*;
        use RedisCommandError::*;

        match v.first() {
            Some(Resp::BulkString(command)) => match *command {
                b"SET" | b"set" | b"Set" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let value = get_bytes_vec(v.get(2))?;

                    Ok(Set(key, value))
                }
                b"SETEX" | b"setex" | b"SetEx" | b"Setex" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let duration = get_bytes_vec(v.get(2)).and_then(|b| parse_duration(b))?;
                    let value = get_bytes_vec(v.get(3))?;
                    let expiry = Expiry::new_from_secs(duration)?;

                    Ok(Setex(key, expiry, value))
                }
                b"MSETNX" | b"MSetnx" | b"msetnx" => {
                    // Draft implementation
                    // Will panic if msetnx has somehow been called with no args, fix later
                    // Maybe do something like split and grab the right partition which should be if it's empty
                    let request = &v[1..]; // [key, value, key, value, key, value, ...] assuming it's even
                    let mut items_vec = Vec::<(Key, Value)>::new();
                    // Now need to map every key value pair into its own iter
                    for key_value in request.chunks(2) {
                        match key_value {
                            [key, value] => {
                                let key = get_bytes_vec(Some(&key))?;
                                let value = get_bytes_vec(Some(&value))?;
                                items_vec.push((key, value));
                            }
                            _ => return Err(ArgNumber),
                        }
                    }
                    Ok(MSetnx(items_vec))
                }
                b"SETNX" | b"setnx" | b"Setnx" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let value = get_bytes_vec(v.get(2))?;

                    Ok(Setnx(key, value))
                }
                b"EXPIRE" | b"expire" | b"Expire" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let duration = get_bytes_vec(v.get(2)).and_then(|b| parse_duration(b))?;
                    let expiry = Expiry::new_from_secs(duration)?;

                    Ok(Expire(key, expiry))
                }
                b"PEXPIRE" | b"Pexpire" | b"PExpire" | b"pexpire" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let duration = get_bytes_vec(v.get(2)).and_then(|b| parse_duration(b))?;
                    let expiry = Expiry::new_from_millis(duration)?;

                    Ok(PExpire(key, expiry))
                }
                b"GET" | b"get" | b"Get" => {
                    let key = get_bytes_vec(v.get(1))?;
                    Ok(Get(key))
                }
                b"GETSET" | b"getset" | b"Getset" | b"GetSet" => {
                    let key = get_bytes_vec(v.get(1))?;
                    let value = get_bytes_vec(v.get(2))?;

                    Ok(GetSet(key, value))
                }
                b"DEL" | b"del" | b"Del" => {
                    let key = get_bytes_vec(v.get(1))?;
                    Ok(Del(key))
                }
                b"INCR" | b"incr" | b"Incr" => {
                    let key = get_bytes_vec(v.get(1))?;
                    Ok(Incr(key))
                }
                b"EXISTS" | b"exists" | b"Exists" => {
                    let key = get_bytes_vec(v.get(1))?;
                    Ok(Exists(key))
                }
                b"INFO" | b"info" | b"Info" => Ok(Info),
                b"PING" | b"ping" | b"Ping" => Ok(Ping),
                b"QUIT" | b"quit" | b"Quit" => Ok(Quit),
                unsupported_command => Err(NotSupported(
                    std::str::from_utf8(unsupported_command)
                        .unwrap()
                        .to_string(),
                )),
            },
            _ => Err(InvalidCommand),
        }
    }
}

#[cfg(test)]
mod tests {
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
            assert_eq!(command, Command::Set(b"mykey".to_vec(), b"value".to_vec()));
        }
    }
}
