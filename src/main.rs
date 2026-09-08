use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::Path;
use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug)]
enum Branch {
    StringValue(String),
    MapValue(BTreeMap<String, Branch>)
}

fn get_element(vec: &[&str], i: usize) -> String {
    if i >= vec.len() {
        return "".into();
    }
    return vec[i].trim().into();
}

fn add_branch(branch: &mut Branch, keys: &str, val: &str) -> Result<(), io::Error> {
    if keys.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "No keys provided."));
    }
    let key: String;
    let subkey: String;
    match keys.find(".") {
        Some(_) => {
            let key_split = keys.splitn(2, ".").collect::<Vec<&str>>();
            key = get_element(&key_split, 0);
            subkey = get_element(&key_split, 1);
        }
        None => {
            key = keys.into();
            subkey = "".into();
        }
    }

    match branch {
        Branch::MapValue(map) => {
            match map.get_mut(&key) {
                Some(sub_branch) => {
                    add_branch(sub_branch, &subkey, val)?;
                }
                None => {
                    if subkey.is_empty() {
                        map.insert(key.into(), Branch::StringValue(val.into()));
                    } else {
                        let mut new_branch = Branch::MapValue(BTreeMap::new());
                        add_branch(&mut new_branch, &subkey, val)?;
                        map.insert(key.into(), new_branch);
                    }
                }
            }
        }
        Branch::StringValue(_) => {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Key {} already exists as a String.", key)));
        }
    }
    Ok(())
}

//   1行読み込んでけｙとvalueのtupleを返す
//   スキーマ読み込みのための布石として分離
fn read_tuple(line: &str, separator: &str) -> Result<(String, String), io::Error> {
    let s = line.trim();
    if s.starts_with("#") || s.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Comment or empty line."));
    }

    let key: String;
    let val: String;
    match s.find(separator) {
        Some(_) => {
            let kv: Vec<&str> = s.splitn(2, separator).collect();
            key = get_element(&kv, 0);
            val = get_element(&kv, 1);
        }
        None => {
            key = s.into();
            val = "".into();
        }
    }
    Ok((key, val))
}

//   confファイルをparseする
fn conf_parser(filepath: &str, mut schema: BTreeMap<String, String>) -> Result<Branch, io::Error> {
    let mut treeroot = Branch::MapValue(BTreeMap::new());

    let reader = BufReader::new(File::open(Path::new(filepath))?);

    for r in reader.lines() {
        match read_tuple(&r?, "=") {
            Ok((key, val)) => {
                //ここにschema_treeのmapvalを参照してvalの型チェックをする機能を入れる(TODO)
                match schema.get_mut(&key) {
                    Some(_) => {
                        let type_str = schema.get(&key).unwrap();
                        match type_str.as_str() {
                            "string" => {
                                //string型は何もしない
                            }
                            "integer" => {
                                if val.parse::<i32>().is_err() {
                                    return Err(io::Error::new(io::ErrorKind::Other, format!("Value for key {} is not an integer.", key)));
                                }
                            }
                            "bool" => {
                                if val.parse::<bool>().is_err() {
                                    return Err(io::Error::new(io::ErrorKind::Other, format!("Value for key {} is not a boolean.", key)));
                                }
                            }
                            _ => {
                                return Err(io::Error::new(io::ErrorKind::Other, format!("Unknown type {} for key {}.", type_str, key)));
                            }
                        }
                    }
                    None => {
                        //schema未定義はノーチェックで通す
                    }
                }
                //ここまで
                add_branch(&mut treeroot, &key, &val)?;
            }
            Err(_) => {
                //コメント行または空行のためスキップ
            }
        }
    }
    return Ok(treeroot);
}

//   schemaファイルをparseする
fn schema_parser(filepath: &str) -> Result<BTreeMap<String, String>, io::Error> {
    let mut m = BTreeMap::new();

    let reader = BufReader::new(File::open(Path::new(filepath))?);

    for r in reader.lines() {
        match read_tuple(&r?, "->") {
            Ok((key, val)) => {
                m.insert(key, val);
            }
            Err(_) => {
                //コメント行または空行のためスキップ
            }
        }
    }
    return Ok(m);
}

//   コマンドライン対応
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: parseconf <conf_file> <schema_file>");
        std::process::exit(1);
    }
    match schema_parser(&args[2]) {
        Ok(schema_tree) => {
            println!("{:?}", schema_tree);
            match conf_parser(&args[1], schema_tree) {
                Ok(conf_tree) => {
                    println!("{:?}", conf_tree);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
